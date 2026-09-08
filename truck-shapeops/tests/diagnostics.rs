use std::error::Error;
use truck_base::diagnostics::{Code, CodedError};
use truck_meshalgo::tessellation::{triangulation_with_diagnostics, try_triangulation};
use truck_modeling::*;
use truck_shapeops::{fillet::*, local::*, *};

fn cube(lo: f64, hi: f64) -> Solid {
    primitive::cuboid(BoundingBox::from_iter([
        Point3::new(lo, lo, lo),
        Point3::new(hi, hi, hi),
    ]))
}
fn edges(solid: &Solid) -> Vec<EdgeID> {
    let mut seen = std::collections::HashSet::new();
    solid
        .edge_iter()
        .map(|e| e.id())
        .filter(|id| seen.insert(*id))
        .collect()
}

#[test]
fn invalid_parameters_preserve_operation_and_nonfinite_values() {
    let solid = cube(0.0, 1.0);
    for (operation, run) in [
        (
            "and",
            try_and::<Curve, Surface>
                as fn(
                    &Solid,
                    &Solid,
                    f64,
                )
                    -> std::result::Result<Solid, truck_base::diagnostics::Diagnostic>,
        ),
        ("or", try_or),
        ("subtract", try_subtract),
    ] {
        for tol in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            let error = run(&solid, &solid, tol).unwrap_err();
            assert_eq!(error.code(), "TRUCK_INVALID_TOLERANCE");
            assert_eq!(error.operation, operation);
            assert_eq!(error.stage, "validate_input");
            assert_eq!(error.context.parameter, Some("tol"));
            assert_eq!(
                error.context.value.as_deref(),
                Some(tol.to_string().as_str())
            );
            let json = serde_json::to_value(&error).unwrap();
            assert_eq!(json["code"], "TRUCK_INVALID_TOLERANCE");
        }
    }
}

#[test]
fn input_topology_preserves_operand_and_coded_source() {
    let bad = Solid::new_unchecked(vec![Shell::default()]);
    let solid = cube(0.0, 1.0);
    let error = try_subtract(&solid, &bad, 0.01).unwrap_err();
    assert_eq!(error.code, Code::InvalidInputTopology);
    assert_eq!(error.context.operand, Some(1));
    assert_eq!(
        error
            .source()
            .unwrap()
            .downcast_ref::<truck_topology::errors::Error>(),
        Some(&truck_topology::errors::Error::EmptyShell)
    );
    let json = serde_json::to_value(&error).unwrap();
    assert_eq!(json["cause"]["code"], "TRUCK_TOPOLOGY_EMPTY_SHELL");
}

#[test]
fn nesting_orientation_and_unrepresentable_results_are_distinct() {
    let outer = cube(0.0, 10.0);
    let inner = cube(2.0, 8.0);
    let invalid = Solid::new([outer.boundaries().clone(), inner.boundaries().clone()].concat());
    let error = try_solid_components(&invalid, 0.01).unwrap_err();
    assert_eq!(error.code, Code::InconsistentOrientation);
    assert_eq!(error.operation, "solid_components");
    assert_eq!(error.context.shell_index, Some(1));
    let mut complement = outer.clone();
    complement.not();
    assert_eq!(
        try_or(&outer, &complement, 0.01).unwrap_err().code,
        Code::UnboundedResult
    );
}

#[test]
fn empty_and_unchanged_results_remain_successful() {
    let a = cube(0.0, 1.0);
    let b = cube(2.0, 3.0);
    assert!(try_and(&a, &b, 0.01).unwrap().boundaries().is_empty());
    assert!(
        !try_subtract_with_effect(&a, &b, 0.01)
            .unwrap()
            .removed_material
    );
    let empty = Solid::new(Vec::new());
    assert!(try_solid_components(&empty, 0.01).unwrap().is_empty());
    assert!(
        !try_subtract_with_effect(&a, &empty, 0.01)
            .unwrap()
            .removed_material
    );
}

#[test]
fn blend_selection_errors_are_actionable_and_do_not_mutate_input() {
    let a = cube(0.0, 2.0);
    let b = cube(3.0, 4.0);
    let before = serde_json::to_string(&a.compress()).unwrap();
    let ids = edges(&a);
    let unknown = try_fillet_solid_edges(&a, &[edges(&b)[0]], 0.2, 0.01).unwrap_err();
    assert_eq!(unknown.code, Code::UnknownEdge);
    assert_eq!(unknown.context.selection_index, Some(0));
    let duplicate = try_fillet_solid_edges(&a, &[ids[0], ids[0]], 0.2, 0.01).unwrap_err();
    assert_eq!(duplicate.code, Code::DuplicateSelection);
    assert_eq!(duplicate.context.selection_index, Some(1));
    let bad_radius = try_fillet_solid_edges(&a, &ids, f64::NAN, 0.01).unwrap_err();
    assert_eq!(bad_radius.code, Code::InvalidParameter);
    assert_eq!(bad_radius.context.parameter, Some("radius"));
    assert_eq!(
        try_fillet_solid_along_wire(&a, &Wire::new(), 0.2, 0.01)
            .unwrap_err()
            .code,
        Code::EmptySelection
    );
    assert_eq!(before, serde_json::to_string(&a.compress()).unwrap());
}

#[test]
fn unsupported_planarity_has_a_face_location() {
    let a = cube(0.0, 2.0);
    let rounded = try_fillet_solid_edges(&a, &edges(&a), 0.2, 0.01)
        .unwrap()
        .solid;
    let error = try_fillet_solid_edges(&rounded, &[edges(&rounded)[0]], 0.05, 0.01).unwrap_err();
    assert_eq!(error.code, Code::NonPlanarFace);
    assert_eq!(error.operation, "fillet_solid_edges");
    assert!(error.context.face_index.is_some());
}

#[test]
fn meshing_reports_all_omissions_and_strict_path_rejects_them() {
    let solid = cube(0.0, 1.0);
    let mut faces: Vec<_> = solid.face_iter().take(2).cloned().collect();
    for face in &mut faces {
        face.set_surface(Surface::Plane(Plane::new(
            Point3::new(0.0, 0.0, 10.0),
            Point3::new(1.0, 0.0, 10.0),
            Point3::new(0.0, 1.0, 10.0),
        )));
    }
    let shell: Shell = faces.into();
    let report = triangulation_with_diagnostics(&shell, 0.01).unwrap();
    assert!(!report.is_complete());
    assert_eq!(report.diagnostics.len(), 2);
    assert_eq!(report.diagnostics[1].context.face_index, Some(1));
    assert_eq!(
        try_triangulation(&shell, 0.01).unwrap_err().code,
        Code::TessellationFailed
    );
    assert_eq!(report.into_complete().unwrap_err().len(), 2);
}

#[test]
fn local_errors_identify_selection_and_parameters() {
    let a = cube(0.0, 1.0);
    let b = cube(2.0, 3.0);
    let id = b.face_iter().next().unwrap().id();
    let error = try_offset_faces(&a, &[id], 0.1).unwrap_err();
    assert_eq!(error.code, Code::UnknownFace);
    assert_eq!(error.context.selection_index, Some(0));
    assert_eq!(error.operation, "offset_faces");
    let error = try_shell(&a, &[], f64::NAN).unwrap_err();
    assert_eq!(error.code, Code::InvalidParameter);
    assert_eq!(error.context.parameter, Some("thickness"));
}

#[derive(Clone, Debug)]
struct NonfiniteDerivative;
impl ScalarFunctionD1 for NonfiniteDerivative {
    fn der_n(&self, n: usize, _: f64) -> f64 {
        match n {
            0 => 0.2,
            1 => f64::NAN,
            _ => 0.0,
        }
    }
}

#[test]
fn variable_radius_failure_identifies_the_derivative_and_station() {
    let solid = cube(0.0, 2.0);
    let wire: Wire = vec![solid.edge_iter().next().unwrap()].into();
    let error = try_fillet_solid_along_wire(&solid, &wire, NonfiniteDerivative, 0.01).unwrap_err();
    assert_eq!(error.code, Code::InvalidParameter);
    assert_eq!(error.context.parameter, Some("radius_derivative"));
    assert_eq!(error.context.value.as_deref(), Some("NaN"));
    assert_eq!(error.context.station, Some(0.0));
    assert_eq!(error.operation, "fillet_solid_along_wire");
}

#[test]
fn local_reintersection_reports_the_actual_unsupported_neighbour() {
    let a = cube(0.0, 2.0);
    let rounded = try_fillet_solid_edges(&a, &edges(&a), 0.2, 0.01)
        .unwrap()
        .solid;
    let planar = rounded.face_iter().next().unwrap();
    let error = try_replace_surfaces(&rounded, &[(planar.id(), planar.surface())]).unwrap_err();
    assert_eq!(error.code, Code::UnsupportedGeometry);
    assert_eq!(error.stage, "validate_surfaces");
    let affected = rounded
        .face_iter()
        .nth(error.context.face_index.unwrap())
        .unwrap();
    assert!(!matches!(
        affected.surface().elementary(),
        Some((
            Elementary::Plane(_) | Elementary::Cylinder { .. } | Elementary::Cone { .. },
            _
        ))
    ));
    assert_eq!(error.context.selection_index, Some(0));
}

#[test]
fn an_empty_surface_intersection_is_success_but_required_missing_boundaries_are_errors() {
    let a = cube(0.0, 2.0);
    let top = a
        .face_iter()
        .find(|f| f.oriented_surface().normal(0.0, 0.0).z > 0.5)
        .unwrap();
    let replacement = Surface::Plane(Plane::new(
        Point3::new(5.0, 0.0, 0.0),
        Point3::new(5.0, 1.0, 0.0),
        Point3::new(5.0, 0.0, 1.0),
    ));
    let error = try_replace_surfaces(&a, &[(top.id(), replacement.clone())]).unwrap_err();
    assert_eq!(error.code, Code::NoIntersection);
    assert_eq!(error.operation, "replace_surfaces");
    assert!(error.context.related_index.is_some());
    let other = replacement.transformed(Matrix4::from_translation(Vector3::unit_x()));
    let result = try_intersect_surfaces(
        &replacement,
        ((0.0, 1.0), (0.0, 1.0)),
        &other,
        ((0.0, 1.0), (0.0, 1.0)),
        0.01,
    )
    .unwrap();
    assert!(result.is_empty());
}
