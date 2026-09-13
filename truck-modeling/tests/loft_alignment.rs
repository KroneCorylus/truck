use truck_modeling::*;

#[test]
fn translated_closed_sections_preserve_correspondence_and_winding() {
    let center = Point3::origin();
    let first: Wire = builder::rsweep(
        &builder::vertex(Point3::new(0.2, 0., 0.)),
        center,
        Vector3::unit_z(),
        Rad(2. * std::f64::consts::PI),
        4,
    );
    for translation in [Vector3::new(2., 0., 10.), Vector3::new(-2., 3., 10.)] {
        let moved = builder::translated(&first, translation);
        for reverse in [false, true] {
            let moved = if reverse {
                moved.inverse()
            } else {
                moved.clone()
            };
            for shift in 0..moved.len() {
                let moved: Wire = moved
                    .iter()
                    .cycle()
                    .skip(shift)
                    .take(moved.len())
                    .cloned()
                    .collect();
                let aligned = builder::align_sections(&[first.clone(), moved]);
                for (a, b) in aligned[0].iter().zip(&aligned[1]) {
                    assert!(a.front().point().near(&(b.front().point() - translation)));
                    assert!(a.back().point().near(&(b.back().point() - translation)));
                }
            }
        }
    }
}
