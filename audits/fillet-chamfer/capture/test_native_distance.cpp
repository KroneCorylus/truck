#define main comparison_main
#include "compare_shapes.cpp"
#undef main
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_Transform.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRepPrimAPI_MakeSphere.hxx>
#include <BRep_Tool.hxx>
#include <Geom_BezierSurface.hxx>
#include <TColgp_Array2OfPnt.hxx>
#include <sstream>

int main(int argc, char** argv) {
  if (argc != 2) return 2;
  int checks = 0, failures = 0;
  auto serialized = [](const TopoDS_Shape& shape) {
    std::ostringstream stream;
    BRepTools::Write(shape, stream);
    return stream.str();
  };
  auto check = [&](const std::string& name, const TopoDS_Shape& source,
                   const TopoDS_Shape& target, double expected, double tolerance) {
    const auto before_source = serialized(source), before_target = serialized(target);
    const auto measured = distance(source, target);
    ++checks;
    const bool valid = measured.samples > 0 && std::isfinite(measured.maximum)
                    && std::abs(measured.maximum - expected) <= tolerance;
    failures += !valid;
    std::cout << std::setprecision(17) << name << " expected=" << expected
              << " measured=" << measured.maximum << " passed=" << valid << '\n';
    ++checks;
    const bool unchanged = before_source == serialized(source) && before_target == serialized(target);
    failures += !unchanged;
    std::cout << name << " inputs_unchanged=" << unchanged << '\n';
  };
  const auto box = BRepPrimAPI_MakeBox(1.,1.,1.).Shape();
  const auto cylinder = BRepPrimAPI_MakeCylinder(1.,2.).Shape();
  const auto sphere = BRepPrimAPI_MakeSphere(1.).Shape();
  check("box identity",box,box,0.,1e-10);
  check("cylinder identity",cylinder,cylinder,0.,1e-10);
  check("sphere identity",sphere,sphere,0.,1e-10);
  const auto shifted = BRepPrimAPI_MakeBox(gp_Pnt(3.,0.,0.),1.,1.,1.).Shape();
  check("separated boxes forward",box,shifted,3.,1e-10);
  check("separated boxes reverse",shifted,box,3.,1e-10);
  const auto outer = BRepPrimAPI_MakeBox(2.,2.,2.).Shape();
  const auto inner = BRepPrimAPI_MakeBox(gp_Pnt(.5,.5,.5),1.,1.,1.).Shape();
  check("inside box to boundary",inner,outer,.5,1e-10);
  check("outside box to inner boundary",outer,inner,std::sqrt(3.)/2.,1e-10);

  TColgp_Array2OfPnt poles(1,2,1,2);
  for (int i=1;i<=2;++i) for (int j=1;j<=2;++j) poles.SetValue(i,j,gp_Pnt(i-1.,j-1.,0.));
  const Handle(Geom_BezierSurface) support = new Geom_BezierSurface(poles);
  const auto trimmed = BRepBuilderAPI_MakeFace(support,0.,.5,0.,1.,1e-12).Face();
  const auto outside = BRepBuilderAPI_MakeFace(support,.75,.8,.4,.6,1e-12).Face();
  check("trimmed spline excludes outside projection",outside,trimmed,.3,1e-10);
  gp_Trsf lift;
  lift.SetTranslation(gp_Vec(0.,0.,.2));
  check("trimmed spline includes boundary distance",
        BRepBuilderAPI_Transform(outside,lift,true).Shape(),trimmed,std::sqrt(.13),1e-10);

  TopoDS_Shape native;
  BRep_Builder builder;
  if (!BRepTools::Read(native,argv[1],builder)) return 3;
  TopoDS_Face face;
  int index = 0;
  for (TopExp_Explorer it(native,TopAbs_FACE);it.More();it.Next()) {
    if (++index == 3) { face = TopoDS::Face(it.Current()); break; }
  }
  if (face.IsNull()) return 4;
  const double u = .20122980253765829, v = .24944293688834626, h = 1e-4;
  for (double a : {u-h,u,u+h}) for (double b : {v-h,v,v+h}) {
    if (BRepClass_FaceClassifier(face,gp_Pnt2d(a,b),1e-9).State() != TopAbs_IN) return 5;
  }
  // The patch is cut from the target's own support, strictly inside its original trim.
  const auto patch = BRepBuilderAPI_MakeFace(BRep_Tool::Surface(face),u-h,u+h,v-h,v+h,1e-12).Face();
  check("native surface subset",patch,native,0.,1e-9);
  for (double scale : {.5,2.5}) {
    gp_Trsf scaling, rigid;
    scaling.SetScale(gp_Pnt(0.,0.,0.),scale);
    rigid.SetRotation(gp_Ax1(gp_Pnt(0.,0.,0.),gp_Dir(1.,2.,3.)),.7);
    rigid.SetTranslationPart(gp_Vec(.3,-.2,.1));
    const auto transform = rigid * scaling;
    check("transformed native subset "+std::to_string(scale),
          BRepBuilderAPI_Transform(patch,transform,true).Shape(),
          BRepBuilderAPI_Transform(native,transform,true).Shape(),0.,scale*1e-9);
  }
  std::cout << checks << " checks, " << failures << " failures\n";
  return failures ? 1 : 0;
}
