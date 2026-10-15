//! Replays a captured physical selection; successful construction is not a parity verdict.
use serde_json::{json, Value};
use std::cell::Cell;
use std::collections::HashSet;
use std::path::Path;
use std::result::Result;
use truck_meshalgo::prelude::*;
use truck_modeling::*;
use truck_shapeops::fillet::*;
use truck_shapeops::SplitClosedEdgesAndFaces;
use truck_stepio::{out::*, r#in::Table};

const TOL: f64 = 0.001;

fn import(path: &str) -> Result<Solid, String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let table = Table::from_step(&text).ok_or("STEP parse failed")?;
    if table.manifold_solid_brep.len() != 1 {
        return Err("Replay currently requires one manifold solid".into());
    }
    let entity = table.manifold_solid_brep.values().next().unwrap();
    let (mut compressed, skipped) = table
        .to_compressed_solid(entity)
        .map_err(|e| format!("{e:?}"))?;
    if !skipped.is_empty() {
        return Err(format!("STEP skipped entities: {skipped:?}"));
    }
    compressed.split_closed_edges_and_faces(TOL / 100.);
    let modeling = compressed
        .try_mapped(
            |p| Some(*p),
            |c| Curve::try_from(c).ok(),
            |s| Surface::try_from(s).ok(),
        )
        .ok_or("Unsupported imported geometry")?;
    Solid::extract(modeling).map_err(|e| format!("{e:?}"))
}

fn main() {
 let s=import(&std::env::args().nth(1).unwrap()).unwrap();
 for (i,f) in s.face_iter().enumerate() {
  for (j,e) in f.boundary_iters().into_iter().flatten().enumerate() {
   let c=e.curve(); let surf=f.surface();
   if !e.is_geometric_consistent() || !surf.include(&c) {
    println!("face {i}, edge {j}, edge_valid {}, include {}, endpoint errors {} {}",e.is_geometric_consistent(),surf.include(&c),c.front().distance(e.absolute_front().point()),c.back().distance(e.absolute_back().point()));
    let (a,b)=c.range_tuple();
    let mut maximum=0.0_f64;let mut exact_failed=0;let mut nearest_failed=0;
    for k in 0..=128 {let p=c.subs(a+(b-a)*k as f64/128.);
     if surf.search_parameter(p,None,100).is_none(){exact_failed+=1;}
     if let Some(uv)=surf.search_nearest_parameter(p,None,100){maximum=maximum.max(surf.subs(uv.0,uv.1).distance(p));}else{nearest_failed+=1;}
    }
    println!("sampled maximum curve/support distance {maximum}, exact failures {exact_failed}/129, nearest failures {nearest_failed}/129");
    let (a,b)=c.range_tuple();
    for t in [a,(a+b)/2.,b] {
     let p=c.subs(t);
     println!("p {p:?} exact {:?} nearest {:?}",surf.search_parameter(p,None,100),surf.search_nearest_parameter(p,None,100).map(|uv| (uv,surf.subs(uv.0,uv.1).distance(p))));
    }
   }
  }
 }
}
