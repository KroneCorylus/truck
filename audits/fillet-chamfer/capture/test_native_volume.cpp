#define main comparison_main
#include "compare_shapes.cpp"
#undef main
#include "blend_capture.cpp"
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRepPrimAPI_MakeSphere.hxx>
#include <BRepBuilderAPI_Transform.hxx>
#include <sstream>
#include <BRepPrimAPI_MakePrism.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <Geom_BSplineCurve.hxx>
#include <TColgp_Array1OfPnt.hxx>
#include <TColStd_Array1OfReal.hxx>
#include <TColStd_Array1OfInteger.hxx>
#include <array>

int main(int argc, char** argv) {
  if (argc != 3) return 2;
  Draw::GetInterpretor().Init();
  int failures = 0, checks = 0;
  auto check = [&](const std::string& name, const TopoDS_Shape& shape, double expected) {
    std::ostringstream before;
    BRepTools::Write(shape, before);
    std::ostringstream out;
    out << std::setprecision(17);
    properties(out, shape);
    const std::string text = out.str();
    const auto start = text.find("\"volume\":");
    if (start == std::string::npos) throw std::runtime_error("capture omitted volume");
    const double captured = std::stod(text.substr(start + 9));
    const std::array<std::pair<const char*, double>, 2> values {{{"capture", captured}, {"comparison", volume(shape)}}};
    for (const auto& item : values) {
      ++checks;
      const bool ok = std::isfinite(item.second) && std::abs(item.second - expected) <= std::abs(expected) * 1e-8;
      failures += !ok;
      std::cout << std::setprecision(17) << name << ' ' << item.first << " expected=" << expected << " actual=" << item.second << " passed=" << ok << '\n';
    }
    std::ostringstream after;
    BRepTools::Write(shape, after);
    ++checks;
    failures += before.str() != after.str();
    std::cout << name << " input_unchanged=" << (before.str() == after.str()) << '\n';
  };
  const double pi = std::acos(-1.);
  check("open face", BRepBuilderAPI_MakeFace(gp_Pln(gp_Pnt(0.,0.,0.),gp_Dir(0.,0.,1.)),0.,1.,0.,1.).Face(), 0.);
  check("box", BRepPrimAPI_MakeBox(2.,3.,4.).Shape(), 24.);
  check("cylinder", BRepPrimAPI_MakeCylinder(2.,3.).Shape(), 12.*pi);
  check("sphere", BRepPrimAPI_MakeSphere(1.).Shape(), 4.*pi/3.);
  const std::array<double, 13> knots {0.,0.,0.,0.,.001,.499,.5,.501,.999,1.,1.,1.,1.};
  const std::array<double, 9> heights {1.,5.,.2,3.,.1,4.,.5,2.,1.};
  TColgp_Array1OfPnt poles(1,9);
  double integral = 0.;
  for (int i=0;i<9;++i) {
    poles.SetValue(i+1, gp_Pnt((knots[i+1]+knots[i+2]+knots[i+3])/3.,0.,heights[i]));
    integral += heights[i]*(knots[i+4]-knots[i])/4.;
  }
  TColStd_Array1OfReal unique(1,7); TColStd_Array1OfInteger multiplicities(1,7);
  for (int i=1;i<=7;++i) {unique.SetValue(i,knots[i+2]);multiplicities.SetValue(i,i==1||i==7?4:1);}
  Handle(Geom_BSplineCurve) curve = new Geom_BSplineCurve(poles,unique,multiplicities,3);
  BRepBuilderAPI_MakeWire wire;
  wire.Add(BRepBuilderAPI_MakeEdge(gp_Pnt(0.,0.,0.),gp_Pnt(1.,0.,0.)));
  wire.Add(BRepBuilderAPI_MakeEdge(gp_Pnt(1.,0.,0.),poles.Value(9)));
  wire.Add(TopoDS::Edge(BRepBuilderAPI_MakeEdge(curve).Edge().Reversed()));
  wire.Add(BRepBuilderAPI_MakeEdge(poles.Value(1),gp_Pnt(0.,0.,0.)));
  const auto spline = BRepPrimAPI_MakePrism(BRepBuilderAPI_MakeFace(wire.Wire()).Face(),gp_Vec(0.,1.,0.)).Shape();
  check("spline exact integral",spline,integral);
  for (int i = 1; i < 3; ++i) {
    TopoDS_Shape shape; BRep_Builder builder;
    if (!BRepTools::Read(shape, argv[i], builder)) return 3;
    const double expected = i == 1 ? 0.0029734793641782342 : 0.002952545217684737;
    check(argv[i], shape, expected);
    gp_Trsf transform;
    transform.SetRotation(gp_Ax1(gp_Pnt(0.,0.,0.), gp_Dir(1.,2.,3.)), 0.7);
    transform.SetTranslationPart(gp_Vec(0.3,-0.2,0.1));
    check(std::string(argv[i])+" transformed", BRepBuilderAPI_Transform(shape, transform, true).Shape(), expected);
  }
  std::cout << checks << " checks, " << failures << " failures\n";
  return failures ? 1 : 0;
}
