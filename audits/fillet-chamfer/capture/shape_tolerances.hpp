#pragma once

#include <BRep_Tool.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <algorithm>
#include <cmath>
#include <ostream>
#include <stdexcept>

namespace blend_audit {
inline void write_shape_tolerances(std::ostream& out, const TopoDS_Shape& shape, const char* name) {
  double face = 0., edge = 0., vertex = 0.;
  auto include = [](double& maximum, double value) {
    if (!std::isfinite(value) || value < 0.) throw std::runtime_error("invalid stored shape tolerance");
    maximum = std::max(maximum, value);
  };
  for (TopExp_Explorer it(shape, TopAbs_FACE); it.More(); it.Next())
    include(face, BRep_Tool::Tolerance(TopoDS::Face(it.Current())));
  for (TopExp_Explorer it(shape, TopAbs_EDGE); it.More(); it.Next())
    include(edge, BRep_Tool::Tolerance(TopoDS::Edge(it.Current())));
  for (TopExp_Explorer it(shape, TopAbs_VERTEX); it.More(); it.Next())
    include(vertex, BRep_Tool::Tolerance(TopoDS::Vertex(it.Current())));
  out << ",\"" << name << "\":{\"face\":" << face << ",\"edge\":" << edge
      << ",\"vertex\":" << vertex << '}';
}
}
