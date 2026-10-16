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
 std::cout<<std::setprecision(17);int index=0;
 for(double scale:{1.,.2}) {
  const double r=5*scale,a=2.5*scale,b=10*scale,height=5*scale,c=std::sqrt(r*r-a*a),pi=std::acos(-1.);
  const gp_Circ circle(gp_Ax2(gp_Pnt(0,0,0),gp_Dir(0,0,1)),r);
  auto arc=BRepBuilderAPI_MakeEdge(circle,-pi/3,4*pi/3).Edge();
  auto bottom=BRepBuilderAPI_MakeEdge(gp_Pnt(-a,-b,0),gp_Pnt(a,-b,0)).Edge();
  auto right=BRepBuilderAPI_MakeEdge(TopExp::LastVertex(bottom),TopExp::FirstVertex(arc)).Edge();
  auto left=BRepBuilderAPI_MakeEdge(TopExp::LastVertex(arc),TopExp::FirstVertex(bottom)).Edge();
  auto wire=BRepBuilderAPI_MakeWire(bottom,right,arc,left).Wire();
  TopoDS_Shape solid=BRepPrimAPI_MakePrism(BRepBuilderAPI_MakeFace(wire).Face(),gp_Vec(0,0,height));
  TopoDS_Face side,top;TopoDS_Edge seed;
  for(TopExp_Explorer it(solid,TopAbs_FACE);it.More();it.Next()) {
   auto f=TopoDS::Face(it.Current());BRepAdaptor_Surface s(f);
   if(s.GetType()!=GeomAbs_Plane)continue;
   if(std::abs(s.Plane().Location().X()+a)<1e-9&&std::abs(s.Plane().Axis().Direction().X())>.99)side=f;
   if(std::abs(s.Plane().Location().Z()-height)<1e-9&&std::abs(s.Plane().Axis().Direction().Z())>.99)top=f;
  }
  if(side.IsNull()||top.IsNull()||!BRepCheck_Analyzer(solid).IsValid())throw std::runtime_error("invalid input");
  for(TopExp_Explorer it(side,TopAbs_EDGE);it.More();it.Next()) {
   auto e=TopoDS::Edge(it.Current());BRepAdaptor_Curve curve(e);auto p=curve.Value((curve.FirstParameter()+curve.LastParameter())/2);
   if(curve.GetType()==GeomAbs_Line&&std::abs(p.X()+a)<1e-9&&std::abs(p.Z()-height)<1e-9)seed=e;
  }
  if(seed.IsNull())throw std::runtime_error("missing seed");
  double expected=height*(5*pi*r*r/6+2*a*b-a*c);
  GProp_GProps props;BRepGProp::VolumeProperties(solid,props);
  if(std::abs(props.Mass()-expected)>1e-8)throw std::runtime_error("input volume mismatch");
  TopoDS_Compound bundle;BRep_Builder builder;builder.MakeCompound(bundle);builder.Add(bundle,solid);builder.Add(bundle,seed);builder.Add(bundle,top);builder.Add(bundle,side);
  if(!BRepTools::Write(bundle,(std::string(argv[1])+"/input-"+std::to_string(index)+".brep").c_str()))throw std::runtime_error("write failed");
  std::cout<<index++<<" scale="<<scale<<" volume="<<props.Mass()<<" expected="<<expected<<std::endl;
 }
}
