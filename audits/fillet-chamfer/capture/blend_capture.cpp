#include "shape_tolerances.hpp"
#include <BRepAdaptor_Curve.hxx>
#include <BRepAdaptor_Surface.hxx>
#include <BRepBuilderAPI_Copy.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepFilletAPI_MakeChamfer.hxx>
#include <BRepFilletAPI_MakeFillet.hxx>
#include <BRepGProp.hxx>
#include <BRepTools.hxx>
#include <DBRep.hxx>
#include <Draw.hxx>
#include <Draw_Interpretor.hxx>
#include <GCPnts_QuasiUniformDeflection.hxx>
#include <GProp_GProps.hxx>
#include <Precision.hxx>
#include <STEPControl_Writer.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <cmath>
#include <cstdlib>
#include <fstream>
#include <iomanip>
#include <stdexcept>
#include <string>
#include <vector>

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
  blend_audit::write_shape_tolerances(out, shape, "stored_tolerances");
}

void edge_geometry(std::ostream& out, const TopoDS_Edge& edge) {
  const BRepAdaptor_Curve curve(edge);
  const GCPnts_QuasiUniformDeflection samples(curve, deflection);
  if (!samples.IsDone() || samples.NbPoints() < 2 || samples.NbPoints() > 100000)
    throw std::runtime_error("edge sampling failed or exceeded capture budget");
  out << "\"curve_type\":" << static_cast<int>(curve.GetType()) << ",\"points\":[";
  for (int j = 1; j <= samples.NbPoints(); ++j) {
    const auto p = samples.Value(j);
    if (!std::isfinite(p.X()) || !std::isfinite(p.Y()) || !std::isfinite(p.Z()))
      throw std::runtime_error("nonfinite selection point");
    if (j != 1) out << ',';
    out << '[' << p.X() << ',' << p.Y() << ',' << p.Z() << ']';
  }
  out << ']';
}

// The runner loads this observer in a fresh DRAW process before any test setup.
struct FilletSettings {
  double angular = 1e-2, approximation3d = 1e-4, parametric = 1e-5, deflection = 1e-3;
  double continuity_angle = 1e-2;
  GeomAbs_Shape continuity = GeomAbs_C1;
  bool valid = true;
} fillet_settings;

int settings(Draw_Interpretor& di, int argc, const char** argv) {
  try {
    if (argc < 2) throw std::runtime_error("missing setting command");
    const std::string command(argv[1]);
    if (argc == 2 && (command == "tolblend" || command == "continuityblend")) return 0;
    auto number = [](const char* value) {
      const double result = Draw::Atof(value);
      if (!std::isfinite(result)) throw std::runtime_error("nonfinite fillet setting");
      return result;
    };
    if (command == "tolblend" && argc == 6) {
      fillet_settings.angular = number(argv[2]);
      fillet_settings.approximation3d = number(argv[3]);
      fillet_settings.parametric = number(argv[4]);
      fillet_settings.deflection = number(argv[5]);
    } else if (command == "continuityblend" && (argc == 3 || argc == 4)) {
      const std::string continuity(argv[2]);
      if (continuity.size() < 2) throw std::runtime_error("invalid continuity token");
      fillet_settings.continuity = continuity[1] == '0' ? GeomAbs_C0
          : continuity[1] == '2' ? GeomAbs_C2 : GeomAbs_C1;
      if (argc == 4) fillet_settings.continuity_angle = std::abs(number(argv[3]));
    } else throw std::runtime_error("unexpected fillet setting arguments");
    return 0;
  } catch (const Standard_Failure& error) {
    di << "Audit settings: " << error.GetMessageString();
  } catch (const std::exception& error) {
    di << "Audit settings: " << error.what();
  }
  fillet_settings.valid = false;
  return 1;
}

int input(Draw_Interpretor& di, int argc, const char** argv) {
  try {
    if (argc < 5) throw std::runtime_error("expected prefix, shape, radius/edge pairs");
    if (!fillet_settings.valid) throw std::runtime_error("native fillet settings were not captured");
    const auto shape = DBRep::Get(argv[2]);
    const std::string prefix = std::string(argv[1]) + ".input";
    export_shape(shape, prefix);
    ChFi3d_FilletShape mode = ChFi3d_Rational;
    if (argc % 2 == 0) {
      const std::string flag(argv[argc - 1]);
      if (flag == "Q" || flag == "q") mode = ChFi3d_QuasiAngular;
      else if (flag == "P" || flag == "p") mode = ChFi3d_Polynomial;
    }
    BRepFilletAPI_MakeFillet builder(shape, mode);
    const auto& config = fillet_settings;
    builder.SetParams(config.angular, 1e-4, config.parametric,
                      config.approximation3d, config.parametric, config.deflection);
    builder.SetContinuity(config.continuity, config.continuity_angle);
    std::vector<std::pair<double, TopoDS_Edge>> seeds;
    for (int i = 3; i + 1 < argc; i += 2) {
      const double radius = Draw::Atof(argv[i]);
      const auto edge = TopoDS::Edge(DBRep::Get(argv[i+1], TopAbs_EDGE));
      if (edge.IsNull() || !std::isfinite(radius)) throw std::runtime_error("invalid radius or edge");
      seeds.emplace_back(radius, edge);
      builder.Add(radius, edge);
    }
    std::ofstream out(prefix + ".json");
    out.exceptions(std::ios::failbit | std::ios::badbit);
    out << std::setprecision(17) << "{\"schema_version\":3,\"operation\":\"fillet\",\"selection_deflection\":" << deflection << ',';
    properties(out, shape);
    out << ",\"native_settings\":{\"representation\":\""
        << (mode == ChFi3d_Rational ? "rational" : mode == ChFi3d_QuasiAngular ? "quasi-angular" : "polynomial")
        << "\",\"angular_tolerance\":" << config.angular
        << ",\"spine_tolerance\":0.0001,\"parametric_tolerance\":" << config.parametric
        << ",\"approximation_3d_tolerance\":" << config.approximation3d
        << ",\"deflection\":" << config.deflection
        << ",\"continuity\":\"" << (config.continuity == GeomAbs_C0 ? "C0" : config.continuity == GeomAbs_C2 ? "C2" : "C1")
        << "\",\"continuity_angle\":" << config.continuity_angle << '}'
        << ",\"selection\":[";
    for (std::size_t i = 0; i < seeds.size(); ++i) {
      if (i != 0) out << ',';
      const auto& seed = seeds[i];
      out << "{\"radius\":" << seed.first << ",\"contour_index\":" << builder.Contour(seed.second) << ',';
      edge_geometry(out, seed.second);
      out << '}';
    }
    out << "],\"contours\":[";
    for (int contour = 1; contour <= builder.NbContours(); ++contour) {
      if (contour != 1) out << ',';
      out << "{\"index\":" << contour << ",\"constant_radius\":";
      if (builder.IsConstant(contour)) {
        const double radius = builder.Radius(contour);
        if (!std::isfinite(radius)) throw std::runtime_error("nonfinite contour radius");
        out << radius;
      } else out << "null";
      out << ",\"edges\":[";
      for (int j = 1; j <= builder.NbEdges(contour); ++j) {
        if (j != 1) out << ',';
        out << '{';
        edge_geometry(out, builder.Edge(contour, j));
        out << '}';
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

void reference_geometry(std::ostream& out, const TopoDS_Face& face) {
  const BRepAdaptor_Surface surface(face);
  out << ",\"reference_plane\":";
  if (surface.GetType() != GeomAbs_Plane) {
    out << "null";
    if (surface.GetType() == GeomAbs_Cylinder) {
      const auto cylinder = surface.Cylinder();
      const auto p = cylinder.Location();
      const auto axis = cylinder.Axis().Direction();
      out << ",\"reference_cylinder\":{\"origin\":[" << p.X() << ',' << p.Y() << ',' << p.Z()
          << "],\"axis\":[" << axis.X() << ',' << axis.Y() << ',' << axis.Z()
          << "],\"radius\":" << cylinder.Radius() << '}';
    }
    return;
  }
  const auto plane = surface.Plane();
  const auto p = plane.Location();
  const auto n = plane.Axis().Direction();
  out << "{\"origin\":[" << p.X() << ',' << p.Y() << ',' << p.Z()
      << "],\"normal\":[" << n.X() << ',' << n.Y() << ',' << n.Z() << "]}";
}

int chamfer_input(Draw_Interpretor& di, int argc, const char** argv) {
  try {
    if (argc < 5) throw std::runtime_error("expected prefix, shape and chamfer selections");
    const auto shape = DBRep::Get(argv[2]);
    const std::string prefix = std::string(argv[1]) + ".input";
    export_shape(shape, prefix);
    std::ofstream out(prefix + ".json");
    out.exceptions(std::ios::failbit | std::ios::badbit);
    out << std::setprecision(17) << "{\"schema_version\":2,\"operation\":\"chamfer\",\"selection_deflection\":" << deflection << ',';
    properties(out, shape);
    BRepFilletAPI_MakeChamfer builder(shape);
    out << ",\"selection\":[";
    for (int i = 3; i < argc;) {
      if (i + 1 >= argc) throw std::runtime_error("incomplete chamfer selection");
      const auto edge = TopoDS::Edge(DBRep::Get(argv[i], TopAbs_EDGE));
      if (edge.IsNull()) throw std::runtime_error("invalid chamfer edge");
      const auto face = DBRep::Get(argv[i + 1], TopAbs_FACE);
      const bool already_selected = builder.Contour(edge) != 0;
      bool valid = false;
      if (i != 3) out << ',';
      if (face.IsNull()) {
        // Native equal-distance chamf uses atof, unlike its other parameter modes.
        const double distance = std::atof(argv[i + 1]);
        if (!std::isfinite(distance)) throw std::runtime_error("nonfinite distance");
        out << "{\"mode\":\"equal-distance\",\"distance\":" << distance;
        valid = distance > Precision::Confusion();
        if (!already_selected && valid) builder.Add(distance, edge);
        i += 2;
      } else {
        if (i + 3 >= argc) throw std::runtime_error("incomplete face chamfer selection");
        const std::string mode(argv[i + 2]);
        if (mode == "A" || mode == "a") {
          if (i + 4 >= argc) throw std::runtime_error("incomplete angle selection");
          const double distance = Draw::Atof(argv[i + 3]), angle = Draw::Atof(argv[i + 4]);
          if (!std::isfinite(distance) || !std::isfinite(angle)) throw std::runtime_error("nonfinite chamfer parameters");
          out << "{\"mode\":\"distance-angle\",\"distance\":" << distance << ",\"angle_degrees\":" << angle;
          const double radians = angle * std::acos(-1.) / 180.;
          valid = distance > Precision::Confusion() && radians > Precision::Confusion()
              && std::acos(-1.) / 2. - radians > Precision::Confusion();
          if (!already_selected && valid) builder.AddDA(distance, radians, edge, TopoDS::Face(face));
          i += 5;
        } else {
          const double d0 = Draw::Atof(argv[i + 2]), d1 = Draw::Atof(argv[i + 3]);
          if (!std::isfinite(d0) || !std::isfinite(d1)) throw std::runtime_error("nonfinite chamfer distances");
          out << "{\"mode\":\"two-distances\",\"distances\":[" << d0 << ',' << d1 << ']';
          valid = d0 > Precision::Confusion() && d1 > Precision::Confusion();
          if (!already_selected && valid) builder.Add(d0, d1, edge, TopoDS::Face(face));
          i += 4;
        }
        reference_geometry(out, TopoDS::Face(face));
      }
      out << ',';
      edge_geometry(out, edge);
      const int contour = builder.Contour(edge);
      const bool applied = !already_selected && valid && contour != 0;
      out << ",\"native_applied\":" << (applied ? "true" : "false") << ",\"ignored_reason\":";
      if (applied) out << "null";
      else if (already_selected) out << "\"already-selected\"";
      else if (!valid) out << "\"invalid-parameters\"";
      else out << "\"not-added\"";
      out << ",\"contour_edges\":[";
      if (applied) {
        for (int j = 1; j <= builder.NbEdges(contour); ++j) {
          if (j != 1) out << ',';
          out << '{';
          edge_geometry(out, builder.Edge(contour, j));
          out << '}';
        }
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
  di.Add("audit_blend_settings", "audit_blend_settings native-command arguments", __FILE__, settings, "Blend audit");
  di.Add("audit_blend_input", "audit_blend_input prefix shape radius edge ...", __FILE__, input, "Blend audit");
  di.Add("audit_chamfer_input", "audit_chamfer_input prefix shape edge parameters ...", __FILE__, chamfer_input, "Blend audit");
  di.Add("audit_blend_output", "audit_blend_output prefix shape", __FILE__, output, "Blend audit");
}
