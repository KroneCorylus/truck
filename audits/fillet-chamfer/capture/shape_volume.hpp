#pragma once
#include <BRepBuilderAPI_Copy.hxx>
#include <BRepCheck_Shell.hxx>
#include <BRepGProp.hxx>
#include <GProp_GProps.hxx>
#include <ShapeUpgrade_ShapeDivideContinuity.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <cmath>
#include <stdexcept>

namespace blend_audit {
inline double shape_volume(const TopoDS_Shape& shape) {
  bool closed = false;
  for (TopExp_Explorer it(shape, TopAbs_SHELL); it.More(); it.Next())
    closed |= BRepCheck_Shell(TopoDS::Shell(it.Current())).Closed(false) == BRepCheck_NoError;
  if (!closed) return 0.;
  // GK splits spline surface spans, but boundary spans on planes/extrusions need explicit cuts.
  ShapeUpgrade_ShapeDivideContinuity split(BRepBuilderAPI_Copy(shape).Shape());
  split.SetBoundaryCriterion(GeomAbs_CN);
  split.SetPCurveCriterion(GeomAbs_CN);
  split.SetSurfaceCriterion(GeomAbs_C0);
  split.SetEdgeMode(2);
  split.SetTolerance(0.);
  split.SetTolerance2d(0.);
  split.Perform();
  if (split.Status(ShapeExtend_FAIL) || split.Result().IsNull())
    throw std::runtime_error("volume boundary splitting failed");
  GProp_GProps properties;
  const double error = BRepGProp::VolumePropertiesGK(split.Result(), properties, 1e-9, true, true);
  if (!std::isfinite(error) || error < 0. || !std::isfinite(properties.Mass()))
    throw std::runtime_error("volume integration failed");
  return properties.Mass();
}
}
