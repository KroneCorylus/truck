#include <BRepAdaptor_Curve.hxx>
#include <BRepBuilderAPI_Copy.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepGProp.hxx>
#include <BRepTools.hxx>
#include <DBRep.hxx>
#include <Draw.hxx>
#include <Draw_Interpretor.hxx>
#include <GCPnts_QuasiUniformDeflection.hxx>
#include <GProp_GProps.hxx>
#include <STEPControl_Writer.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <cmath>
#include <fstream>
#include <iomanip>
#include <stdexcept>
#include <string>

namespace {
constexpr double deflection = 0.00001;

void export_shape(const TopoDS_Shape& shape, const std::string& prefix) {
  if (shape.IsNull()) throw std::runtime_error("shape is null");
  if (!BRepTools::Write(shape, (prefix + ".brep").c_str()))
    throw std::runtime_error("BREP write failed");
  BRepBuilderAPI_Copy copy(shape);
  STEPControl_Writer writer;
  if (writer.Transfer(copy.Shape(), STEPControl_AsIs) != IFSelect_RetDone
      || writer.Write((prefix + ".step").c_str()) != IFSelect_RetDone)
    throw std::runtime_error("STEP write failed");
}

void properties(std::ostream& out, const TopoDS_Shape& shape) {
  int solids = 0;
  for (TopExp_Explorer it(shape, TopAbs_SOLID); it.More(); it.Next()) ++solids;
  GProp_GProps volume, area;
  BRepGProp::VolumeProperties(shape, volume, 1e-9, true, false);
  BRepGProp::SurfaceProperties(shape, area, 1e-9);
  if (!std::isfinite(volume.Mass()) || !std::isfinite(area.Mass()))
    throw std::runtime_error("nonfinite shape properties");
  out << "\"native_valid\":" << (BRepCheck_Analyzer(shape).IsValid() ? "true" : "false")
      << ",\"solids\":" << solids << ",\"volume\":" << volume.Mass() << ",\"area\":" << area.Mass();
}

int input(Draw_Interpretor& di, int argc, const char** argv) {
  try {
    if (argc < 5 || (argc - 3) % 2 != 0)
      throw std::runtime_error("expected prefix, shape, radius/edge pairs");
    const auto shape = DBRep::Get(argv[2]);
    const std::string prefix = std::string(argv[1]) + ".input";
    export_shape(shape, prefix);
    std::ofstream out(prefix + ".json");
    out.exceptions(std::ios::failbit | std::ios::badbit);
    out << std::setprecision(17) << "{\"schema_version\":1,\"operation\":\"fillet\",\"selection_deflection\":" << deflection << ',';
    properties(out, shape);
    out << ",\"selection\":[";
    for (int i = 3; i + 1 < argc; i += 2) {
      const double radius = Draw::Atof(argv[i]);
      const auto edge = TopoDS::Edge(DBRep::Get(argv[i+1], TopAbs_EDGE));
      if (edge.IsNull() || !std::isfinite(radius)) throw std::runtime_error("invalid radius or edge");
      const BRepAdaptor_Curve curve(edge);
      const GCPnts_QuasiUniformDeflection samples(curve, deflection);
      if (!samples.IsDone() || samples.NbPoints() < 2 || samples.NbPoints() > 100000)
        throw std::runtime_error("edge sampling failed or exceeded capture budget");
      if (i != 3) out << ',';
      out << "{\"radius\":" << radius << ",\"curve_type\":" << static_cast<int>(curve.GetType()) << ",\"points\":[";
      for (int j = 1; j <= samples.NbPoints(); ++j) {
        const auto p = samples.Value(j);
        if (!std::isfinite(p.X()) || !std::isfinite(p.Y()) || !std::isfinite(p.Z()))
          throw std::runtime_error("nonfinite selection point");
        if (j != 1) out << ',';
        out << '[' << p.X() << ',' << p.Y() << ',' << p.Z() << ']';
      }
      out << "]}";
    }
    out << "]}\n";
    return 0;
  } catch (const Standard_Failure& error) {
    di << "Audit capture: " << error.GetMessageString();
  } catch (const std::exception& error) {
    di << "Audit capture: " << error.what();
  }
  return 1;
}

int output(Draw_Interpretor& di, int argc, const char** argv) {
  try {
    if (argc != 3) throw std::runtime_error("expected prefix and result shape");
    const auto shape = DBRep::Get(argv[2]);
    const std::string prefix = std::string(argv[1]) + ".result";
    export_shape(shape, prefix);
    std::ofstream out(prefix + ".json");
    out.exceptions(std::ios::failbit | std::ios::badbit);
    out << std::setprecision(17) << '{';
    properties(out, shape);
    out << "}\n";
    return 0;
  } catch (const Standard_Failure& error) {
    di << "Audit capture: " << error.GetMessageString();
  } catch (const std::exception& error) {
    di << "Audit capture: " << error.what();
  }
  return 1;
}
}

extern "C" Standard_EXPORT void PLUGINFACTORY(Draw_Interpretor& di) {
  di.Add("audit_blend_input", "audit_blend_input prefix shape radius edge ...", __FILE__, input, "Blend audit");
  di.Add("audit_blend_output", "audit_blend_output prefix shape", __FILE__, output, "Blend audit");
}
