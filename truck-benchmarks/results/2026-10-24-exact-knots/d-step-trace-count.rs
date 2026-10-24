use truck_stepio::{r#in::{Table, step_geometry::*},out::*};
use truck_stepio::r#in::ruststep::tables::IntoOwned;
fn main(){
 let text=std::fs::read_to_string("truck-stepio/tests/fixtures/spline-extrusion-pcurves/C1-equal-006.step").unwrap();
 let table=Table::from_step(&text).unwrap();
 let (solid,skipped)=table.to_compressed_solid(table.manifold_solid_brep.values().next().unwrap()).unwrap();
 println!("shells={} skipped={:?}",solid.boundaries.len(),skipped);
 let owned=table.pcurve.get(&2224).unwrap().clone().into_owned(&table).unwrap();
 let original=PCurve::try_from(&owned).unwrap();
 let imported=&solid.boundaries[0].edges[10].curve;
 let (a,b)=imported.range_tuple();
 let mut maxima=[0.0_f64;3];
 for i in 0..=128 {let t=a+(b-a)*i as f64/128.; maxima[0]=maxima[0].max(imported.subs(t).distance(original.subs(t)));maxima[1]=maxima[1].max(imported.der(t).distance(original.der(t)));maxima[2]=maxima[2].max(imported.der2(t).distance(original.der2(t)));}
 println!("raw master pcurve2224 range={:?} maximum position/der1/der2 differences from importededge10={:?}",original.range_tuple(),maxima);
 for shell in &solid.boundaries {
  println!("edges={} faces={}",shell.edges.len(),shell.faces.len());
  for (i,e) in shell.edges.iter().enumerate(){
   let kind=match &e.curve {Curve3D::PCurve(p)=>match p.curve().as_ref(){Curve2D::BSplineCurve(_)=>"spline pcurve",_=>"other pcurve"},Curve3D::BSplineCurve(_)=>"spatial spline",_=>"other"};
   println!("edge={} kind={} range={:?} ends={:?}/{:?}",i,kind,e.curve.range_tuple(),e.curve.front(),e.curve.back());
  }
 }
}
