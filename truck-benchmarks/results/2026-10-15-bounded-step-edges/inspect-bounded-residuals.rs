use ruststep::tables::EntityTable;
use truck_stepio::r#in::{step_geometry::*, EdgeCurveHolder, Table};
fn main() {
    let path=std::env::args().nth(1).unwrap();
    let table=Table::from_step(&std::fs::read_to_string(path).unwrap()).unwrap();
    let mut ids: Vec<_>=table.edge_curve.keys().copied().collect();ids.sort();
    for id in ids {
        let edge=EntityTable::<EdgeCurveHolder>::get_owned(&table,id).unwrap();
        if let Err(error)=edge.parse_curve3d() {
            println!("edge {id}: {error:?}");
            let raw=Curve3D::try_from(&edge.edge_geometry).unwrap();
            let (p,q)=(Point3::from(&edge.edge_start.vertex_geometry),Point3::from(&edge.edge_end.vertex_geometry));
            let (p,q)=if edge.same_sense {(p,q)} else {(q,p)};
            println!("range {:?}, full endpoint errors {} {}",raw.range_tuple(),raw.front().distance(p),raw.back().distance(q));
            for point in [p,q] {
                println!("exact {:?}, nearest {:?}",raw.search_parameter(point,None,100),raw.search_nearest_parameter(point,None,100).map(|t|(t,raw.subs(t).distance(point))));
            }
        }
    }
}
