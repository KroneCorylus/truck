use truck_stepio::r#in::{step_geometry::*, Table};
fn main() {
    for path in std::env::args().skip(1) {
        let table = Table::from_step(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let (solid, skipped) = table
            .to_compressed_solid(table.manifold_solid_brep.values().next().unwrap())
            .unwrap();
        assert!(skipped.is_empty());
        for shell in solid.boundaries {
            for (index, face) in shell.faces.iter().enumerate() {
                if !matches!(
                    face.surface,
                    Surface::ElementarySurface(ElementarySurface::ToroidalSurface(_))
                ) {
                    continue;
                }
                let (mut total, mut failed) = (0, 0);
                let mut first = None;
                for edge in face.boundaries.iter().flatten() {
                    let curve = &shell.edges[edge.index].curve;
                    for p in curve.parameter_division(curve.range_tuple(), 0.00001).1 {
                        total += 1;
                        if face.surface.search_parameter(p, None, 100).is_none() {
                            failed += 1;
                            if first.is_none() {
                                first = Some(p);
                            }
                        }
                    }
                }
                println!(
                    "input={path} face={index} samples={total} rejected={failed} first={first:?}"
                );
            }
        }
    }
}
