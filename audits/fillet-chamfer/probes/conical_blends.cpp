#include <BRepAdaptor_Surface.hxx>
#include <BRepBuilderAPI_Copy.hxx>
#include <BRepBuilderAPI_MakeSolid.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepFilletAPI_MakeChamfer.hxx>
#include <BRepFilletAPI_MakeFillet.hxx>
#include <BRepGProp.hxx>
#include <BRepPrimAPI_MakeCone.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepTools.hxx>
#include <BRep_Tool.hxx>
#include <GProp_GProps.hxx>
#include <Standard_Version.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Solid.hxx>
#include <cmath>
#include <iomanip>
#include <iostream>
#include <stdexcept>

int main() {
  std::cout << std::setprecision(15) << "OCCT " << OCC_VERSION_COMPLETE << '\n';
  const double pi = std::acos(-1.);
  for (bool cavity : {false, true}) {
    for (double slope : {-0.25, 0.25}) {
      const double top_radius = 5. + 8. * slope, q = std::sqrt(1. + slope * slope);
      const double input_volume = pi * 8. / 3. * (25. + 5. * top_radius + top_radius * top_radius);
      const auto frustum = BRepPrimAPI_MakeCone(5., top_radius, 8.).Solid();
      const auto input = [&]() {
        if (!cavity) return frustum;
        const auto outer = BRepPrimAPI_MakeBox(gp_Pnt(-12., -12., -1.), gp_Pnt(12., 12., 10.)).Solid();
        BRepBuilderAPI_MakeSolid make;
        make.Add(TopoDS::Shell(TopExp_Explorer(outer, TopAbs_SHELL).Current()));
        auto inner = TopoDS::Shell(TopExp_Explorer(frustum, TopAbs_SHELL).Current());
        inner.Reverse();
        make.Add(inner);
        return make.Solid();
      }();
      GProp_GProps properties;
      BRepGProp::VolumeProperties(input, properties);
      if (!BRepCheck_Analyzer(input).IsValid() || std::abs(properties.Mass() - (cavity ? 6336. - input_volume : input_volume)) > 1e-6)
        throw std::runtime_error("invalid cone fixture");
      TopoDS_Face top;
      for (TopExp_Explorer it(input, TopAbs_FACE); it.More(); it.Next()) {
        const auto face = TopoDS::Face(it.Current());
        const BRepAdaptor_Surface surface(face);
        if (surface.GetType() == GeomAbs_Plane && std::abs(surface.Plane().Location().Z() - 8.) < 1e-8)
          top = face;
      }
      if (top.IsNull()) throw std::runtime_error("missing top face");
      const auto rim = TopoDS::Edge(TopExp_Explorer(top, TopAbs_EDGE).Current());
      for (bool is_fillet : {true, false}) {
        std::cout << "cavity=" << cavity << " slope=" << slope << " operation=" << (is_fillet ? "fillet" : "chamfer");
        try {
          BRepBuilderAPI_Copy copy(input);
          const auto edge = TopoDS::Edge(copy.ModifiedShape(rim));
          TopoDS_Shape result;
          double expected;
          if (is_fillet) {
            BRepFilletAPI_MakeFillet operation(copy.Shape());
            operation.Add(0.5, edge);
            operation.Build();
            if (!operation.IsDone()) { std::cout << " done=false\n"; continue; }
            result = operation.Shape();
            const double r = 0.5, beta = std::acos(-slope / q);
            const double area = r * r * (q + slope - beta / 2);
            const double center = top_radius - r * (slope + q);
            const double polygon_moment = r * r * r * (q + slope) * (2 * (q + slope) + 1 / q) / 6;
            const double sector_moment = r * r * r * (1 + slope / q) / 3;
            expected = input_volume - 2 * pi * (center * area + polygon_moment - sector_moment);
          } else {
            BRepFilletAPI_MakeChamfer operation(copy.Shape());
            operation.Add(0.5, 0.3, edge, TopoDS::Face(copy.ModifiedShape(top)));
            operation.Build();
            if (!operation.IsDone()) { std::cout << " done=false\n"; continue; }
            result = operation.Shape();
            const double area = 0.5 * 0.5 * 0.3 / q;
            const double centroid = top_radius - (0.5 + slope * 0.3 / q) / 3;
            expected = input_volume - 2 * pi * area * centroid;
          }
          if (cavity) expected = 6336. - expected;
          bool dimensions_match = false;
          int solids = 0;
          for (TopExp_Explorer it(result, TopAbs_SOLID); it.More(); it.Next()) ++solids;
          for (TopExp_Explorer it(result, TopAbs_FACE); it.More(); it.Next()) {
            const auto face = TopoDS::Face(it.Current());
            const BRepAdaptor_Surface surface(face);
            double a, b, c, d;
            BRepTools::UVBounds(face, a, b, c, d);
            bool matches = true;
            for (double u : {0.25, 0.5, 0.75}) {
              for (double v : {0.25, 0.5, 0.75}) {
                const auto p = surface.Value(a + (b - a) * u, c + (d - c) * v);
                const double rho = std::hypot(p.X(), p.Y());
                const double residual = is_fillet
                    ? std::hypot(rho - top_radius + 0.5 * (slope + q), p.Z() - 7.5) - 0.5
                    : (rho + (0.5 * q / 0.3 - slope) * (p.Z() - 8.) - top_radius + 0.5)
                          / std::hypot(1., 0.5 * q / 0.3 - slope);
                matches &= std::abs(residual) < 1e-7;
              }
            }
            dimensions_match |= matches;
          }
          BRepGProp::VolumeProperties(result, properties, 1e-9, true, false);
          std::cout << " done=true valid=" << BRepCheck_Analyzer(result).IsValid() << " solids=" << solids
                    << " volume=" << properties.Mass() << " expected_volume=" << expected
                    << " dimensions_match=" << dimensions_match
                    << " volume_matches=" << (std::abs(properties.Mass() - expected) < 1e-6) << '\n';
        } catch (const Standard_Failure& error) {
          std::cout << " exception=" << error.GetMessageString() << '\n';
        }
      }
    }
  }
}
