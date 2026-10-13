use ruststep::{
    ast::{EntityInstance, Name, Parameter},
    tables::EntityTable,
};
use std::f64::consts::{PI, TAU};
use truck_modeling::*;
use truck_stepio::{
    out::*,
    r#in::{Table, VertexPointHolder},
};

fn latitude(angle: f64, start: f64) -> Wire {
    let center = Point3::new(0., 0., angle.cos());
    let vertex = builder::vertex(center + angle.sin() * Vector3::new(start.cos(), start.sin(), 0.));
    builder::rsweep(&vertex, center, Vector3::unit_z(), Rad(TAU), 3)
}

fn exported_poles(shell: &Shell) -> Vec<Point3> {
    let compressed = shell.compress();
    let step = StepDisplay::new(
        Default::default(),
        StepDesign::from_model(StepModel::from(&compressed)),
    )
    .to_string();
    let table = Table::from_step(&step).unwrap();
    let (read, skipped) = table
        .to_compressed_shell(table.shell.values().next().unwrap())
        .unwrap();
    assert!(skipped.is_empty(), "{skipped:?}");
    assert_eq!(read.vertices.len(), compressed.vertices.len());
    assert_eq!(read.edges.len(), compressed.edges.len());
    assert_eq!(read.faces.len(), compressed.faces.len());
    let read = Shell::extract(
        read.try_mapped(
            |p| Some(*p),
            |c| Curve::try_from(c).ok(),
            |s| Surface::try_from(s).ok(),
        )
        .unwrap(),
    )
    .unwrap();
    assert!(read.is_geometric_consistent());
    for (source, read) in shell.iter().zip(&read) {
        let source_boundaries = source.boundaries();
        let read_boundaries = read.boundaries();
        assert_eq!(source_boundaries.len(), read_boundaries.len());
        for (source, read) in source_boundaries.iter().zip(&read_boundaries) {
            assert_eq!(source.len(), read.len());
            for (source, read) in source.edge_iter().zip(read.edge_iter()) {
                assert!(source.front().point().near(&read.front().point()));
                assert!(source.back().point().near(&read.back().point()));
                let (source, read) = (source.oriented_curve(), read.oriented_curve());
                let (a, b) = source.range_tuple();
                for i in 0..=32 {
                    let t = a + (b - a) * i as f64 / 32.;
                    let point = source.subs(t);
                    let u = read.search_parameter(point, None, 50).unwrap();
                    assert!(read.subs(u).near(&point));
                    assert!(source.der(t).normalize().dot(read.der(u).normalize()) > 1. - 1e-8);
                }
            }
        }
    }
    ruststep::parser::parse(&step)
        .unwrap()
        .data
        .iter()
        .flat_map(|data| &data.entities)
        .filter_map(|entity| {
            let EntityInstance::Simple { record, .. } = entity else {
                return None;
            };
            if record.name != "VERTEX_LOOP" {
                return None;
            }
            let Parameter::List(parameters) = &record.parameter else {
                panic!("vertex loop parameters")
            };
            let Parameter::Ref(Name::Entity(id)) = parameters[1] else {
                panic!("vertex loop reference")
            };
            let vertex = EntityTable::<VertexPointHolder>::get_owned(&table, id).unwrap();
            Some(Point3::from(&vertex.vertex_geometry))
        })
        .collect()
}

#[test]
fn enclosed_poles_follow_boundary_winding_and_surface_parameters() {
    let transform = Matrix4::from_translation(Vector3::new(13., -7., 23.))
        * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
        * Matrix4::from_scale(2.);
    for north in [false, true] {
        for reversed_surface in [false, true] {
            for reversed_face in [false, true] {
                for start in [0., 0.37, 2.1] {
                    let mut wire = latitude(if north { 0.7 } else { PI - 0.7 }, start);
                    if !north {
                        wire.invert();
                    }
                    let mut surface =
                        Surface::Sphere(Processor::new(Sphere::new(Point3::origin(), 1.)));
                    if reversed_surface {
                        wire.invert();
                        surface.invert();
                    }
                    let mut face = Face::new(vec![wire], surface);
                    if reversed_face {
                        face.invert();
                    }
                    let face = builder::transformed(&face, transform);
                    let shell = Shell::from(vec![face]);
                    assert!(shell.is_geometric_consistent());
                    let poles = exported_poles(&shell);
                    assert_eq!(poles.len(), 1, "north={north}, reversed_surface={reversed_surface}, reversed_face={reversed_face}, start={start}");
                    assert!(poles[0].near(&transform.transform_point(Point3::new(
                        0.,
                        0.,
                        if north { 1. } else { -1. }
                    ))));
                }
            }
        }
    }
}

#[test]
fn annular_faces_do_not_gain_a_polar_boundary() {
    let surface = Surface::Sphere(Processor::new(Sphere::new(Point3::origin(), 1.)));
    let face = Face::new(
        vec![latitude(1.2, 0.37), latitude(0.7, 0.37).inverse()],
        surface,
    );
    assert!(exported_poles(&Shell::from(vec![face])).is_empty());
}
