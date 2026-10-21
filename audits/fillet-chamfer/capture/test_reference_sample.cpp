#include "blend_capture.cpp"
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_Transform.hxx>
#include <BRepPrimAPI_MakePrism.hxx>
#include <Geom_BezierCurve.hxx>
#include <sstream>
#include <array>

int main(int argc, char** argv) {
  if (argc != 2) return 2;
  Draw::GetInterpretor().Init();
  int checks = 0, failures = 0, index = 0;
  for (double curvature : {.8, -.4}) {
    TColgp_Array1OfPnt poles(1,3);
    poles(1)=gp_Pnt(0.,0.,1.); poles(2)=gp_Pnt(.5,0.,1.); poles(3)=gp_Pnt(1.,0.,1.+curvature);
    Handle(Geom_BezierCurve) curve = new Geom_BezierCurve(poles);
    BRepBuilderAPI_MakeWire wire;
    wire.Add(BRepBuilderAPI_MakeEdge(gp_Pnt(0.,0.,0.),poles(1)));
    wire.Add(BRepBuilderAPI_MakeEdge(curve));
    wire.Add(BRepBuilderAPI_MakeEdge(poles(3),gp_Pnt(1.,0.,0.)));
    wire.Add(BRepBuilderAPI_MakeEdge(gp_Pnt(1.,0.,0.),gp_Pnt(0.,0.,0.)));
    const auto input = BRepPrimAPI_MakePrism(BRepBuilderAPI_MakeFace(wire.Wire()).Face(),gp_Vec(0.,1.,0.)).Shape();
    for (bool moved : {false, true}) {
      gp_Trsf transform;
      if (moved) {
        transform.SetRotation(gp_Ax1(gp_Pnt(0.,0.,0.),gp_Dir(1.,2.,3.)),.7);
        transform.SetTranslationPart(gp_Vec(.3,-.2,.1));
      }
      const auto shape=BRepBuilderAPI_Transform(input,transform,true).Shape();
      std::ostringstream before; BRepTools::Write(shape,before);
      TopoDS_Edge guide;
      gp_Pnt a(0.,0.,1.), b(0.,1.,1.);a.Transform(transform);b.Transform(transform);
      for (TopExp_Explorer it(shape,TopAbs_EDGE);it.More();it.Next()) {
        const auto edge=TopoDS::Edge(it.Current());const BRepAdaptor_Curve c(edge);
        const auto p=c.Value(c.FirstParameter()),q=c.Value(c.LastParameter());
        if ((p.Distance(a)<1e-8&&q.Distance(b)<1e-8)||(p.Distance(b)<1e-8&&q.Distance(a)<1e-8)) { guide=edge;break; }
      }
      if (guide.IsNull()) return 3;
      for (TopExp_Explorer it(shape,TopAbs_FACE);it.More();it.Next()) {
        const auto face=TopoDS::Face(it.Current());
        if (BRepAdaptor_Surface(face).GetType()==GeomAbs_Plane) continue;
        std::ostringstream reference; reference<<std::setprecision(17);reference_geometry(reference,face);
        ++checks;const bool captured=reference.str().find("\"reference_sample\":{")!=std::string::npos;
        failures+=!captured;
        std::cout<<"curvature="<<curvature<<" moved="<<moved<<" sample_captured="<<captured<<'\n';
        const std::string prefix=std::string(argv[1])+"/"+std::to_string(index++);
        export_shape(shape,prefix+".input");
        std::ofstream out(prefix+".input.json");out<<std::setprecision(17);
        out<<"{\"schema_version\":2,\"operation\":\"chamfer\",\"selection\":[{\"mode\":\"two-distances\",\"distances\":[0.3,0.2]"<<reference.str()<<',';
        edge_geometry(out,guide);out<<",\"native_applied\":true,\"ignored_reason\":null,\"contour_edges\":[{";edge_geometry(out,guide);out<<"}]}]}\n";
      }
      std::ostringstream after;BRepTools::Write(shape,after);
      ++checks;failures+=before.str()!=after.str();
    }
  }
  std::cout<<checks<<" checks, "<<failures<<" failures\n";
  std::cout.flush();
  std::_Exit(failures ? 1 : 0);
}
