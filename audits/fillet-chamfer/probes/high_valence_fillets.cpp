#include <BRepAdaptor_Surface.hxx>
#include <BRepBuilderAPI_Copy.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <BRepBuilderAPI_MakeSolid.hxx>
#include <BRepBuilderAPI_Sewing.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepFilletAPI_MakeFillet.hxx>
#include <BRepGProp.hxx>
#include <BRep_Tool.hxx>
#include <GProp_GProps.hxx>
#include <Standard_Version.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopTools_IndexedMapOfShape.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Solid.hxx>
#include <cmath>
#include <iomanip>
#include <iostream>
#include <stdexcept>
#include <vector>

double volume(int count, double apothem, double height, double radius) {
  const double pi = std::acos(-1.), alpha = pi / count, slope = apothem / height;
  const double hypotenuse = std::sqrt(1. + slope * slope), nz = slope / hypotenuse;
  const double axis_z = 1. / std::sqrt(1. + std::pow(slope / std::cos(alpha), 2));
  const double beta = std::acos(std::cos(2 * alpha) / (hypotenuse * hypotenuse) + nz * nz);
  const double center_z = height - radius / nz, contact_z = center_z + radius * nz;
  const double planar_area = count * contact_z * contact_z * slope * std::tan(alpha) * hypotenuse;
  const double middle_z = nz / std::cos(beta / 2);
  const double integral_z = 2 * middle_z * std::sin(beta / 2);
  const double integral_z2 = middle_z * middle_z * (beta + std::sin(beta)) / 2;
  const double cylinder_area = count * radius / axis_z * (center_z * beta + radius * integral_z);
  const double face_angle = 2 * std::asin(slope * std::tan(alpha) * axis_z);
  const double sphere_area = radius * radius * (2 * pi - count * face_angle);
  const double base_area = nz * planar_area
      + count * radius / axis_z * (center_z * integral_z + radius * integral_z2)
      + radius * radius * count * axis_z * beta / 2;
  return (radius * (planar_area + cylinder_area + sphere_area) + center_z * base_area) / 3;
}

int main() {
  std::cout << std::setprecision(15) << "OCCT " << OCC_VERSION_COMPLETE << '\n';
  for (int count : {4, 5, 6, 8}) {
    const double pi = std::acos(-1.), circumradius = 5. / std::cos(pi / count);
    std::vector<gp_Pnt> points;
    for (int i = 0; i < count; ++i)
      points.emplace_back(circumradius * std::cos(2 * pi * i / count),
                          circumradius * std::sin(2 * pi * i / count), 0.);
    points.emplace_back(0., 0., 10.);
    BRepBuilderAPI_Sewing sewing(1e-8);
    BRepBuilderAPI_MakePolygon base;
    for (int i = count - 1; i >= 0; --i) base.Add(points[i]);
    base.Close();
    sewing.Add(BRepBuilderAPI_MakeFace(base.Wire()).Face());
    for (int i = 0; i < count; ++i) {
      BRepBuilderAPI_MakePolygon polygon;
      for (int k : {i, (i + 1) % count, count}) polygon.Add(points[k]);
      polygon.Close();
      sewing.Add(BRepBuilderAPI_MakeFace(polygon.Wire()).Face());
    }
    sewing.Perform();
    const auto input = BRepBuilderAPI_MakeSolid(TopoDS::Shell(sewing.SewedShape())).Solid();
    GProp_GProps properties;
    BRepGProp::VolumeProperties(input, properties);
    const double expected_input = count * 25. * std::tan(pi / count) * 10. / 3.;
    if (!BRepCheck_Analyzer(input).IsValid() || std::abs(properties.Mass() - expected_input) > 1e-6)
      throw std::runtime_error("invalid pyramid fixture");
    TopTools_IndexedMapOfShape edges;
    TopExp::MapShapes(input, TopAbs_EDGE, edges);
    std::vector<TopoDS_Edge> ridges;
    for (int i = 1; i <= edges.Extent(); ++i) {
      const auto edge = TopoDS::Edge(edges(i));
      TopoDS_Vertex a, b;
      TopExp::Vertices(edge, a, b);
      if (std::abs(BRep_Tool::Pnt(a).Z() - BRep_Tool::Pnt(b).Z()) > 1.) ridges.push_back(edge);
    }
    if (static_cast<int>(ridges.size()) != count) throw std::runtime_error("missing pyramid ridges");
    for (double radius : {0.2, 1.2}) {
      std::cout << "ridges=" << count << " radius=" << radius;
      try {
        BRepBuilderAPI_Copy copy(input);
        BRepFilletAPI_MakeFillet fillet(copy.Shape());
        for (const auto& ridge : ridges)
          fillet.Add(radius, TopoDS::Edge(copy.ModifiedShape(ridge)));
        fillet.Build();
        if (!fillet.IsDone()) {
          std::cout << " done=false\n";
          continue;
        }
        const auto result = fillet.Shape();
        BRepGProp::VolumeProperties(result, properties, 1e-9, true, false);
        int solids = 0, faces = 0, matching_spheres = 0;
        for (TopExp_Explorer it(result, TopAbs_SOLID); it.More(); it.Next()) ++solids;
        for (TopExp_Explorer it(result, TopAbs_FACE); it.More(); it.Next()) {
          ++faces;
          const BRepAdaptor_Surface surface(TopoDS::Face(it.Current()));
          if (surface.GetType() == GeomAbs_Sphere) {
            const auto sphere = surface.Sphere();
            if (std::abs(sphere.Radius() - radius) < 1e-8
                && sphere.Location().Distance(gp_Pnt(0., 0., 10. - radius * std::sqrt(5.))) < 1e-8)
              ++matching_spheres;
          }
        }
        const double expected = volume(count, 5., 10., radius);
        std::cout << " done=true valid=" << BRepCheck_Analyzer(result).IsValid()
                  << " solids=" << solids << " faces=" << faces << " matching_spheres=" << matching_spheres
                  << " volume=" << properties.Mass() << " spherical_corner_volume=" << expected
                  << " matches_spherical_volume=" << (std::abs(properties.Mass() - expected) < 1e-6) << '\n';
      } catch (const Standard_Failure& error) {
        std::cout << " exception=" << error.GetMessageString() << '\n';
      }
    }
  }
}
