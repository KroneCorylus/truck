#include <BRepAdaptor_Surface.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepFilletAPI_MakeFillet.hxx>
#include <BRepGProp.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepTools.hxx>
#include <BRep_Tool.hxx>
#include <GProp_GProps.hxx>
#include <Standard_Version.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <cmath>
#include <iomanip>
#include <iostream>
#include <stdexcept>

int main() {
  std::cout << std::setprecision(15) << "OCCT " << OCC_VERSION_COMPLETE << '\n';
  for (double height : {0.2, 1., 10.}) {
    for (double radius : {0.5, 10.}) {
      const auto input = BRepPrimAPI_MakeBox(100., 100., height).Solid();
      TopoDS_Edge selected;
      for (TopExp_Explorer it(input, TopAbs_EDGE); it.More(); it.Next()) {
        bool on_axis = true;
        int vertices = 0;
        for (TopExp_Explorer v(it.Current(), TopAbs_VERTEX); v.More(); v.Next()) {
          const auto p = BRep_Tool::Pnt(TopoDS::Vertex(v.Current()));
          on_axis &= std::hypot(p.X(), p.Y()) < 1e-8;
          ++vertices;
        }
        if (vertices == 2 && on_axis) selected = TopoDS::Edge(it.Current());
      }
      if (selected.IsNull() || !BRepCheck_Analyzer(input).IsValid())
        throw std::runtime_error("invalid fixture or selection");
      std::cout << "height=" << height << " radius=" << radius;
      try {
        BRepFilletAPI_MakeFillet operation(input);
        operation.Add(radius, selected);
        operation.Build();
        if (!operation.IsDone()) { std::cout << " done=false\n"; continue; }
        const auto result = operation.Shape();
        int solids = 0;
        bool dimensions = false;
        for (TopExp_Explorer it(result, TopAbs_SOLID); it.More(); it.Next()) ++solids;
        for (TopExp_Explorer it(result, TopAbs_FACE); it.More(); it.Next()) {
          const auto face = TopoDS::Face(it.Current());
          const BRepAdaptor_Surface surface(face);
          if (surface.GetType() != GeomAbs_Cylinder) continue;
          double a, b, c, d;
          BRepTools::UVBounds(face, a, b, c, d);
          bool matches = true;
          for (int i = 0; i <= 16; ++i) {
            for (int j = 0; j <= 8; ++j) {
              const auto p = surface.Value(a + (b-a)*i/16., c + (d-c)*j/8.);
              matches &= std::abs(std::hypot(p.X()-radius, p.Y()-radius)-radius) < 1e-7
                         && p.Z() >= -1e-7 && p.Z() <= height + 1e-7;
            }
          }
          dimensions |= matches;
        }
        GProp_GProps props;
        BRepGProp::VolumeProperties(result, props, 1e-9, true, false);
        const double expected = height * (10000 - (1-std::acos(-1.)/4)*radius*radius);
        std::cout << " done=true valid=" << BRepCheck_Analyzer(result).IsValid() << " solids=" << solids
                  << " dimensions_match=" << dimensions << " volume=" << props.Mass()
                  << " expected_volume=" << expected << " volume_error=" << std::abs(props.Mass()-expected) << '\n';
      } catch (const Standard_Failure& error) {
        std::cout << " exception=" << error.GetMessageString() << '\n';
      }
    }
  }
}
