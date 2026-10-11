#include <BRepAdaptor_Surface.hxx>
#include <BRepBuilderAPI_Copy.hxx>
#include <BRepBuilderAPI_MakeSolid.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepFilletAPI_MakeFillet.hxx>
#include <BRepGProp.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeSphere.hxx>
#include <BRepTools.hxx>
#include <GProp_GProps.hxx>
#include <Standard_Version.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Solid.hxx>
#include <algorithm>
#include <cmath>
#include <iomanip>
#include <iostream>
#include <limits>
#include <stdexcept>

double volume(double top, double r) {
  const auto sphere = [](double z) { return 25 * z - z * z * z / 3; };
  if (r == 0) return std::acos(-1.) * (sphere(top) - sphere(-4));
  const double zc = top - r, rc = std::sqrt((5 - r) * (5 - r) - zc * zc);
  const double zs = 5 * zc / (5 - r);
  const auto arc = [&](double u) {
    return (rc * rc + r * r) * u - u * u * u / 3
        + rc * (u * std::sqrt(std::max(0., r * r - u * u))
                + r * r * std::asin(std::clamp(u / r, -1., 1.)));
  };
  return std::acos(-1.) * (sphere(zs) - sphere(-4) + arc(r) - arc(zs - zc));
}

int main() {
  std::cout << std::setprecision(15) << "OCCT " << OCC_VERSION_COMPLETE << '\n';
  for (bool cavity : {false, true}) {
    for (double top_z : {-3., 3.}) {
      const auto band = BRepPrimAPI_MakeSphere(5., std::asin(-4. / 5), std::asin(top_z / 5)).Solid();
      const auto input = [&]() {
        if (!cavity) return band;
        const auto outer = BRepPrimAPI_MakeBox(gp_Pnt(-8., -8., -8.), gp_Pnt(8., 8., 8.)).Solid();
        BRepBuilderAPI_MakeSolid make;
        make.Add(TopoDS::Shell(TopExp_Explorer(outer, TopAbs_SHELL).Current()));
        auto inner = TopoDS::Shell(TopExp_Explorer(band, TopAbs_SHELL).Current());
        inner.Reverse();
        make.Add(inner);
        return make.Solid();
      }();
      GProp_GProps props;
      BRepGProp::VolumeProperties(input, props);
      const double original_volume = cavity ? 4096 - volume(top_z, 0) : volume(top_z, 0);
      if (!BRepCheck_Analyzer(input).IsValid() || std::abs(props.Mass() - original_volume) > 1e-6)
        throw std::runtime_error("invalid sphere band fixture");
      TopoDS_Face top;
      for (TopExp_Explorer it(input, TopAbs_FACE); it.More(); it.Next()) {
        auto face = TopoDS::Face(it.Current());
        BRepAdaptor_Surface surface(face);
        if (surface.GetType() == GeomAbs_Plane && std::abs(surface.Plane().Location().Z() - top_z) < 1e-8)
          top = face;
      }
      if (top.IsNull()) throw std::runtime_error("missing top face");
      const auto rim = TopoDS::Edge(TopExp_Explorer(top, TopAbs_EDGE).Current());
      std::cout << "cavity=" << cavity << " top=" << top_z;
      try {
        BRepBuilderAPI_Copy copy(input);
        BRepFilletAPI_MakeFillet operation(copy.Shape());
        operation.Add(0.5, TopoDS::Edge(copy.ModifiedShape(rim)));
        operation.Build();
        if (!operation.IsDone()) { std::cout << " done=false\n"; continue; }
        const auto result = operation.Shape();
        const double zc = top_z - 0.5, rc = std::sqrt(4.5 * 4.5 - zc * zc);
        double best_patch_error = std::numeric_limits<double>::infinity();
        int solids = 0;
        for (TopExp_Explorer it(result, TopAbs_SOLID); it.More(); it.Next()) ++solids;
        for (TopExp_Explorer it(result, TopAbs_FACE); it.More(); it.Next()) {
          const auto face = TopoDS::Face(it.Current());
          const BRepAdaptor_Surface surface(face);
          double a, b, c, d;
          BRepTools::UVBounds(face, a, b, c, d);
          double error = 0.;
          for (int i = 0; i <= 16; ++i) {
            for (int j = 0; j <= 16; ++j) {
              const auto p = surface.Value(a + (b - a) * i / 16., c + (d - c) * j / 16.);
              error = std::max(error, std::abs(std::hypot(std::hypot(p.X(), p.Y()) - rc, p.Z() - zc) - 0.5));
            }
          }
          best_patch_error = std::min(best_patch_error, error);
        }
        BRepGProp::VolumeProperties(result, props, 1e-9, true, false);
        const double expected = cavity ? 4096 - volume(top_z, 0.5) : volume(top_z, 0.5);
        std::cout << " done=true valid=" << BRepCheck_Analyzer(result).IsValid() << " solids=" << solids
                  << " volume=" << props.Mass() << " expected_volume=" << expected
                  << " sampled_radius_error=" << best_patch_error
                  << " volume_error=" << std::abs(props.Mass() - expected) << '\n';
      } catch (const Standard_Failure& error) {
        std::cout << " exception=" << error.GetMessageString() << '\n';
      }
    }
  }
}
