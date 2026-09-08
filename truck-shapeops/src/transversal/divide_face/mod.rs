#![allow(clippy::many_single_char_names)]

use super::faces_classification::FacesClassification;
use super::loops_store::*;
use rustc_hash::FxHashMap as HashMap;
use std::ops::Deref;
use std::result::Result;
use truck_base::diagnostics::{Code, Diagnostic};
use truck_meshalgo::prelude::*;
use truck_topology::*;

fn create_parameter_boundary<P, C, S>(
    face: &Face<P, C, S>,
    wire: &Wire<P, C>,
    polys: &mut HashMap<EdgeID<C>, PolylineCurve<P>>,
    tol: f64,
) -> Option<PolylineCurve<Point2>>
where
    P: Copy,
    C: BoundedCurve<Point = P> + ParameterDivision1D<Point = P>,
    S: ParametricSurface<Point = P> + SearchParameter<D2, Point = P>,
{
    let surface = face.surface();
    let pt = wire.front_vertex()?.point();
    let p: Point2 = surface.search_parameter(pt, None, 100)?.into();
    let vec = wire.edge_iter().try_fold(vec![p], |mut vec, edge| {
        let poly = polys.entry(edge.id()).or_insert_with(|| {
            let curve = edge.curve();
            let div = curve.parameter_division(curve.range_tuple(), tol).1;
            PolylineCurve(div)
        });
        let mut p = *vec.last().unwrap();
        let closure = |q: &P| -> Option<Point2> {
            let mut next: Point2 = surface
                .search_parameter(*q, Some(p.into()), 100)
                .or_else(|| surface.search_parameter(*q, None, 100))?
                .into();
            for (i, period) in [surface.u_period(), surface.v_period()]
                .into_iter()
                .enumerate()
            {
                if let Some(period) = period {
                    next[i] += ((p[i] - next[i]) / period).round() * period;
                }
            }
            p = next;
            Some(p)
        };
        let add: Option<Vec<Point2>> = match edge.orientation() {
            true => poly.iter().skip(1).map(closure).collect(),
            false => poly.iter().rev().skip(1).map(closure).collect(),
        };
        vec.append(&mut add?);
        Some(vec)
    })?;
    Some(PolylineCurve(vec))
}

#[derive(Clone, Debug)]
struct WireChunk<'a, C> {
    poly: PolylineCurve<Point2>,
    wire: &'a BoundaryWire<Point3, C>,
}

type FaceWithShapesOpStatus<C, S> = (Face<Point3, C, S>, ShapesOpStatus);
fn divide_one_face<C, S>(
    face: &Face<Point3, C, S>,
    loops: &Loops<Point3, C>,
    tol: f64,
) -> Result<Vec<FaceWithShapesOpStatus<C, S>>, Diagnostic>
where
    C: BoundedCurve<Point = Point3> + ParameterDivision1D<Point = Point3>,
    S: ParametricSurface3D + SearchParameter<D2, Point = Point3>,
{
    let (mut pre_faces, mut negative_wires) = (Vec::new(), Vec::new());
    let mut map = HashMap::default();
    loops.iter().try_for_each(|wire| {
        let poly = create_parameter_boundary(face, wire, &mut map, tol)
            .ok_or_else(|| Diagnostic::new(Code::ProjectionFailed, "boolean", "divide_faces"))?;
        match poly.area() > 0.0 {
            true => pre_faces.push(vec![WireChunk { poly, wire }]),
            false => negative_wires.push(WireChunk { poly, wire }),
        }
        Ok(())
    })?;
    negative_wires.into_iter().try_for_each(|chunk| {
        let pt = chunk.poly.front();
        let op = pre_faces
            .iter_mut()
            .find(|face| face[0].poly.include(pt))
            .ok_or_else(|| Diagnostic::new(Code::FaceDivisionFailed, "boolean", "divide_faces"))?;
        op.push(chunk);
        Ok(())
    })?;
    pre_faces
        .into_iter()
        .map(|pre_face| {
            let surface = face.surface();
            let op = pre_face
                .iter()
                .find(|chunk| chunk.wire.status() != ShapesOpStatus::Unknown);
            let status = match op {
                Some(chunk) => chunk.wire.status(),
                None => ShapesOpStatus::Unknown,
            };
            let wires: Vec<Wire<Point3, C>> = pre_face
                .into_iter()
                .map(|chunk| chunk.wire.deref().clone())
                .collect();
            let mut new_face = Face::try_new(wires, surface).map_err(|e| {
                Diagnostic::new(Code::InvalidOutputTopology, "boolean", "divide_faces")
                    .with_coded_source(e)
            })?;
            if !face.orientation() {
                new_face.invert();
            }
            Ok((new_face, status))
        })
        .collect()
}

#[cfg(test)]
pub fn divide_faces<C, S>(
    shell: &Shell<Point3, C, S>,
    loops_store: &LoopsStore<Point3, C>,
    tol: f64,
) -> Option<FacesClassification<Point3, C, S>>
where
    C: BoundedCurve<Point = Point3> + ParameterDivision1D<Point = Point3>,
    S: ParametricSurface3D + SearchParameter<D2, Point = Point3>,
{
    try_divide_faces(shell, loops_store, tol).ok()
}

pub fn try_divide_faces<C, S>(
    shell: &Shell<Point3, C, S>,
    loops_store: &LoopsStore<Point3, C>,
    tol: f64,
) -> Result<FacesClassification<Point3, C, S>, Diagnostic>
where
    C: BoundedCurve<Point = Point3> + ParameterDivision1D<Point = Point3>,
    S: ParametricSurface3D + SearchParameter<D2, Point = Point3>,
{
    let mut res = FacesClassification::<Point3, C, S>::default();
    shell
        .iter()
        .zip(loops_store)
        .enumerate()
        .try_for_each(|(index, (face, loops))| {
            if loops
                .iter()
                .all(|wire| wire.status() == ShapesOpStatus::Unknown)
            {
                // The loops carry the edges as split and merged by the cuts of other faces.
                let wires = loops.iter().map(|wire| wire.deref().clone()).collect();
                let mut new_face = Face::try_new(wires, face.surface()).map_err(|e| {
                    Diagnostic::new(Code::InvalidOutputTopology, "boolean", "divide_faces")
                        .face(index)
                        .with_coded_source(e)
                })?;
                if !face.orientation() {
                    new_face.invert();
                }
                res.push(new_face, ShapesOpStatus::Unknown);
            } else {
                let vec = divide_one_face(face, loops, tol).map_err(|e| e.face(index))?;
                vec.into_iter()
                    .for_each(|(face, status)| res.push(face, status));
            }
            Ok(())
        })?;
    Ok(res)
}

#[cfg(test)]
mod tests;
