#include <BRepAdaptor_Surface.hxx>
#include <BRepBuilderAPI_Copy.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <BRepBuilderAPI_MakeSolid.hxx>
#include <BRepBuilderAPI_Sewing.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepFilletAPI_MakeChamfer.hxx>
#include <BRepGProp.hxx>
#include <BRepTools.hxx>
#include <BRep_Tool.hxx>
#include <GProp_GProps.hxx>
#include <Standard_Version.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Solid.hxx>
#include <array>
#include <cmath>
#include <iomanip>
#include <iostream>
#include <stdexcept>
#include <utility>
#include <vector>

int main() {
  const std::array<gp_Pnt, 5> points = {
      gp_Pnt(0, 0, 0), gp_Pnt(10, 0, 0), gp_Pnt(10, 10, 0),
      gp_Pnt(0, 10, 0), gp_Pnt(5, 5, 10)};
  BRepBuilderAPI_Sewing sewing(1e-8);
  for (const auto& indices : std::vector<std::vector<int>>{
           {3, 2, 1, 0}, {0, 1, 4}, {1, 2, 4}, {2, 3, 4}, {3, 0, 4}}) {
    BRepBuilderAPI_MakePolygon polygon;
    for (int i : indices) polygon.Add(points[i]);
    polygon.Close();
    sewing.Add(BRepBuilderAPI_MakeFace(polygon.Wire()).Face());
  }
  sewing.Perform();
  const auto input = BRepBuilderAPI_MakeSolid(TopoDS::Shell(sewing.SewedShape())).Solid();
  GProp_GProps properties;
  BRepGProp::VolumeProperties(input, properties);
  if (!BRepCheck_Analyzer(input).IsValid() || std::abs(properties.Mass() - 1000. / 3.) > 1e-6)
    throw std::runtime_error("invalid pyramid fixture");

  std::array<TopoDS_Edge, 4> ridges;
  for (TopExp_Explorer it(input, TopAbs_EDGE); it.More(); it.Next()) {
    const auto edge = TopoDS::Edge(it.Current());
    TopoDS_Vertex a, b;
    TopExp::Vertices(edge, a, b);
    auto p = BRep_Tool::Pnt(a), q = BRep_Tool::Pnt(b);
    if (std::abs(p.Z() - q.Z()) < 1.) continue;
    if (p.Z() > q.Z()) std::swap(p, q);
    const int corner = p.Y() < 5 ? (p.X() < 5 ? 0 : 1) : (p.X() < 5 ? 3 : 2);
    ridges[corner] = edge;
  }
  for (const auto& edge : ridges)
    if (edge.IsNull()) throw std::runtime_error("missing pyramid ridge");

  std::cout << std::setprecision(15) << "OCCT " << OCC_VERSION_COMPLETE << '\n';
  const std::array<std::pair<int, int>, 4> corners = {{{-1, -1}, {1, -1}, {1, 1}, {-1, 1}}};
  const std::array<double, 5> correction = {1. / 4., 7. / 9., 2. / 3., 4. / 3., 2.};
  const std::array<std::vector<int>, 5> selections = {{{0}, {0, 1}, {0, 2}, {0, 1, 2}, {0, 1, 2, 3}}};
  for (double distance : {0.2, 1.2}) {
    for (std::size_t k = 0; k < selections.size(); ++k) {
      const auto& selected = selections[k];
      const double h = distance * std::sqrt(6. / 5.);
      const double expected = 1000. / 3. - selected.size() * 5. * h * h + correction[k] * h * h * h;
      std::cout << "distance=" << distance << " ridges=";
      for (int i : selected) std::cout << i;
      try {
        BRepBuilderAPI_Copy copy(input);
        BRepFilletAPI_MakeChamfer chamfer(copy.Shape());
        for (int i : selected) chamfer.Add(distance, TopoDS::Edge(copy.ModifiedShape(ridges[i])));
        chamfer.Build();
        if (!chamfer.IsDone()) {
          std::cout << " done=false\n";
          continue;
        }
        const auto result = chamfer.Shape();
        BRepGProp::VolumeProperties(result, properties, 1e-9, true, false);
        bool planes_match = true;
        for (int i : selected) {
          const auto [x, y] = corners[i];
          const gp_Dir wanted(x, y, 1);
          const double offset = (5 * x + 5 * y + 10 - h) / std::sqrt(3.);
          bool found = false;
          for (TopExp_Explorer it(result, TopAbs_FACE); it.More(); it.Next()) {
            const auto face = TopoDS::Face(it.Current());
            const BRepAdaptor_Surface surface(face);
            double a, b, c, d;
            BRepTools::UVBounds(face, a, b, c, d);
            gp_Pnt p;
            gp_Vec du, dv;
            surface.D1((a + b) / 2, (c + d) / 2, p, du, dv);
            auto cross = du.Crossed(dv);
            if (cross.Magnitude() < 1e-12) continue;
            gp_Dir normal(cross);
            if (face.Orientation() == TopAbs_REVERSED) normal.Reverse();
            bool match = normal.Dot(wanted) > 1. - 1e-8;
            for (double u : {0.25, 0.5, 0.75}) {
              for (double v : {0.25, 0.5, 0.75}) {
                const auto point = surface.Value(a + (b - a) * u, c + (d - c) * v);
                match &= std::abs(wanted.XYZ().Dot(point.XYZ()) - offset) < 1e-8;
              }
            }
            found |= match;
          }
          planes_match &= found;
        }
        int solids = 0, faces = 0;
        for (TopExp_Explorer it(result, TopAbs_SOLID); it.More(); it.Next()) ++solids;
        for (TopExp_Explorer it(result, TopAbs_FACE); it.More(); it.Next()) ++faces;
        const bool valid = BRepCheck_Analyzer(result).IsValid();
        const bool volume_match = std::abs(properties.Mass() - expected) <= 1e-6;
        std::cout << " done=true valid=" << valid << " solids=" << solids << " faces=" << faces
                  << " volume=" << properties.Mass() << " truck_miter_volume=" << expected
                  << " planes_match=" << planes_match << " matches_miter_volume=" << volume_match << '\n';
      } catch (const Standard_Failure& error) {
        std::cout << " exception=" << error.GetMessageString() << '\n';
      }
    }
  }
}
