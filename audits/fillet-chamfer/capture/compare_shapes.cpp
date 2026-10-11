#include <BRepAdaptor_Curve.hxx>
#include <BRepAdaptor_Surface.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepClass_FaceClassifier.hxx>
#include <BRepExtrema_DistShapeShape.hxx>
#include <BRepGProp.hxx>
#include <BRepTools.hxx>
#include <BRep_Builder.hxx>
#include <GProp_GProps.hxx>
#include <STEPControl_Reader.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <cmath>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <stdexcept>
#include <string>

namespace {
struct Residual {
  double maximum = 0.;
  int samples = 0;
};

Residual distance(const TopoDS_Shape& source, const TopoDS_Shape& target) {
  Residual result;
  TopoDS_Compound boundary;
  BRep_Builder builder;
  builder.MakeCompound(boundary);
  for (TopExp_Explorer it(target, TopAbs_FACE); it.More(); it.Next())
    builder.Add(boundary, it.Current());
  auto measure = [&](const gp_Pnt& point) {
    const BRepExtrema_DistShapeShape query(BRepBuilderAPI_MakeVertex(point), boundary);
    if (!query.IsDone() || !std::isfinite(query.Value()))
      throw std::runtime_error("distance query failed");
    result.maximum = std::max(result.maximum, query.Value());
    ++result.samples;
  };
  for (TopExp_Explorer it(source, TopAbs_EDGE); it.More(); it.Next()) {
    const BRepAdaptor_Curve curve(TopoDS::Edge(it.Current()));
    const double a = curve.FirstParameter(), b = curve.LastParameter();
    for (int i = 0; i <= 24; ++i) measure(curve.Value(a + (b - a) * i / 24.));
  }
  for (TopExp_Explorer it(source, TopAbs_FACE); it.More(); it.Next()) {
    const auto face = TopoDS::Face(it.Current());
    const BRepAdaptor_Surface surface(face);
    double u0, u1, v0, v1;
    BRepTools::UVBounds(face, u0, u1, v0, v1);
    if (!std::isfinite(u0) || !std::isfinite(u1) || !std::isfinite(v0) || !std::isfinite(v1))
      throw std::runtime_error("unbounded sample domain");
    for (int i = 0; i <= 10; ++i) {
      for (int j = 0; j <= 10; ++j) {
        const double u = u0 + (u1 - u0) * i / 10., v = v0 + (v1 - v0) * j / 10.;
        const auto state = BRepClass_FaceClassifier(face, gp_Pnt2d(u, v), 1e-8).State();
        if (state == TopAbs_IN || state == TopAbs_ON) measure(surface.Value(u, v));
      }
    }
  }
  if (!result.samples) throw std::runtime_error("empty sample set");
  return result;
}

int solids(const TopoDS_Shape& shape) {
  int count = 0;
  for (TopExp_Explorer it(shape, TopAbs_SOLID); it.More(); it.Next()) ++count;
  return count;
}

double volume(const TopoDS_Shape& shape) {
  GProp_GProps properties;
  BRepGProp::VolumeProperties(shape, properties, 1e-9, true, false);
  if (!std::isfinite(properties.Mass())) throw std::runtime_error("nonfinite volume");
  return properties.Mass();
}
}

int main(int argc, char** argv) {
  if (argc != 4) {
    std::cerr << "usage: compare_shapes native.brep truck.step output.json\n";
    return 2;
  }
  try {
    TopoDS_Shape native;
    BRep_Builder builder;
    if (!BRepTools::Read(native, argv[1], builder)) throw std::runtime_error("BREP read failed");
    STEPControl_Reader reader;
    if (reader.ReadFile(argv[2]) != IFSelect_RetDone || !reader.TransferRoots())
      throw std::runtime_error("STEP read failed");
    const auto truck = reader.OneShape();
    if (native.IsNull() || truck.IsNull()) throw std::runtime_error("empty imported shape");
    const auto forward = distance(native, truck), reverse = distance(truck, native);
    std::ofstream out(argv[3]);
    out.exceptions(std::ios::failbit | std::ios::badbit);
    out << std::setprecision(17)
        << "{\"schema_version\":1,\"comparison\":\"sampled-only\","
        << "\"native_valid\":" << (BRepCheck_Analyzer(native).IsValid() ? "true" : "false")
        << ",\"truck_step_valid\":" << (BRepCheck_Analyzer(truck).IsValid() ? "true" : "false")
        << ",\"native_solids\":" << solids(native) << ",\"truck_solids\":" << solids(truck)
        << ",\"native_volume\":" << volume(native) << ",\"truck_volume\":" << volume(truck)
        << ",\"native_to_truck_max_distance\":" << forward.maximum
        << ",\"native_to_truck_samples\":" << forward.samples
        << ",\"truck_to_native_max_distance\":" << reverse.maximum
        << ",\"truck_to_native_samples\":" << reverse.samples << "}\n";
    return 0;
  } catch (const Standard_Failure& error) {
    std::cerr << error.GetMessageString() << '\n';
  } catch (const std::exception& error) {
    std::cerr << error.what() << '\n';
  }
  return 1;
}
