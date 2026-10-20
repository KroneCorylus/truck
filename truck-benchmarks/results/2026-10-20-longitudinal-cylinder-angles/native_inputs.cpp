#include <BRepAdaptor_Curve.hxx>
#include <BRepAdaptor_Surface.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRepPrimAPI_MakePrism.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRepAlgoAPI_Cut.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepGProp.hxx>
#include <BRepTools.hxx>
#include <BRep_Builder.hxx>
#include <GProp_GProps.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Compound.hxx>
#include <gp_Circ.hxx>
#include <cmath>
#include <iostream>
#include <iomanip>
#include <stdexcept>
#include <string>
int main(int argc,char**argv) {
 if(argc!=2)return 1;
 std::cout<<std::setprecision(17);
 int index=0;
 for(bool convex:{true,false})for(double offset:{-2.,0.,2.}) {
  const double r=5.,height=10.,x=std::sqrt(r*r-offset*offset),theta=std::asin(offset/r);
  const gp_Circ circle(gp_Ax2(gp_Pnt(0,0,0),gp_Dir(0,0,1)),r);
  TopoDS_Shape solid;
  if(convex) {
   auto arc=BRepBuilderAPI_MakeEdge(circle,theta,std::acos(-1.)-theta).Edge();
   auto chord=BRepBuilderAPI_MakeEdge(TopExp::LastVertex(arc),TopExp::FirstVertex(arc)).Edge();
   auto wire=BRepBuilderAPI_MakeWire(arc,chord).Wire();
   solid=BRepPrimAPI_MakePrism(BRepBuilderAPI_MakeFace(wire).Face(),gp_Vec(0,0,height));
  } else solid=BRepAlgoAPI_Cut(BRepPrimAPI_MakeBox(gp_Pnt(-8,offset,0),16,8-offset,height).Shape(),BRepPrimAPI_MakeCylinder(r,height).Shape()).Shape();
  TopoDS_Face plane,cylinder;TopoDS_Edge seed;
  for(TopExp_Explorer it(solid,TopAbs_FACE);it.More();it.Next()) {
   auto f=TopoDS::Face(it.Current());BRepAdaptor_Surface s(f);
   if(s.GetType()==GeomAbs_Cylinder)cylinder=f;
   if(s.GetType()==GeomAbs_Plane&&std::abs(s.Plane().Location().Y()-offset)<1e-9&&std::abs(s.Plane().Axis().Direction().Y())>.99)plane=f;
  }
  if(plane.IsNull()||cylinder.IsNull()||!BRepCheck_Analyzer(solid).IsValid())throw std::runtime_error("invalid input");
  for(TopExp_Explorer it(plane,TopAbs_EDGE);it.More();it.Next()) {
   auto e=TopoDS::Edge(it.Current());BRepAdaptor_Curve c(e);auto p=c.Value((c.FirstParameter()+c.LastParameter())/2);
   if(c.GetType()==GeomAbs_Line&&std::abs(p.X()-x)<1e-9&&std::abs(p.Z()-height/2)<1e-9)seed=e;
  }
  if(seed.IsNull())throw std::runtime_error("missing seed");
  double area=r*r*std::acos(offset/r)-offset*x;
  double expected=(convex?area:16*(8-offset)-area)*height;
  GProp_GProps props;BRepGProp::VolumeProperties(solid,props);
  if(std::abs(props.Mass()-expected)>1e-8)throw std::runtime_error("input volume mismatch");
  TopoDS_Compound bundle;BRep_Builder b;b.MakeCompound(bundle);b.Add(bundle,solid);b.Add(bundle,seed);b.Add(bundle,plane);b.Add(bundle,cylinder);
  if(!BRepTools::Write(bundle,(std::string(argv[1])+"/input-"+std::to_string(index)+".brep").c_str()))throw std::runtime_error("write failed");
  std::cout<<index++<<" convex="<<convex<<" offset="<<offset<<" volume="<<props.Mass()<<" expected="<<expected<<std::endl;
 }
}
