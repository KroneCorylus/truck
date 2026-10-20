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
#include <BRepAlgoAPI_Common.hxx>
#include <BRepAlgoAPI_Fuse.hxx>
int main(int argc,char**argv) {
 if(argc!=2)return 1;
 std::cout<<std::setprecision(17);
 for(int index=0;index<3;++index) {
  const double r1=5.,r2=index==1?4.:5.,d=6.,height=10.;
  const double x=(r1*r1-r2*r2+d*d)/(2*d),y=std::sqrt(r1*r1-x*x);
  const bool joined=index==2;
  auto a=BRepPrimAPI_MakeCylinder(gp_Ax2(gp_Pnt(0,0,0),gp_Dir(0,0,1)),r1,height).Shape();
  auto b=BRepPrimAPI_MakeCylinder(gp_Ax2(gp_Pnt(d,0,0),gp_Dir(0,0,1)),r2,height).Shape();
  TopoDS_Shape solid=joined?BRepAlgoAPI_Fuse(a,b).Shape():BRepAlgoAPI_Common(a,b).Shape();
  TopoDS_Face first,second;TopoDS_Edge seed;
  for(TopExp_Explorer it(solid,TopAbs_FACE);it.More();it.Next()) {
   auto f=TopoDS::Face(it.Current());BRepAdaptor_Surface surface(f);
   if(surface.GetType()!=GeomAbs_Cylinder)continue;
   for(TopExp_Explorer edges(f,TopAbs_EDGE);edges.More();edges.Next()) {
    auto e=TopoDS::Edge(edges.Current());BRepAdaptor_Curve c(e);auto p=c.Value((c.FirstParameter()+c.LastParameter())/2);
    if(c.GetType()==GeomAbs_Line&&std::abs(p.X()-x)<1e-8&&std::abs(p.Y()-y)<1e-8) {
     seed=e;
     if(std::abs(surface.Cylinder().Location().X())<1e-8)first=f;else second=f;
    }
   }
  }
  if(first.IsNull()||second.IsNull()||seed.IsNull()||!BRepCheck_Analyzer(solid).IsValid())throw std::runtime_error("invalid fixture");
  double area=r1*r1*std::acos(x/r1)+r2*r2*std::acos((d-x)/r2)-d*y;
  if(joined)area=std::acos(-1.)*(r1*r1+r2*r2)-area;
  GProp_GProps props;BRepGProp::VolumeProperties(solid,props);
  if(std::abs(props.Mass()-area*height)>1e-7)throw std::runtime_error("volume mismatch");
  TopoDS_Compound bundle;BRep_Builder builder;builder.MakeCompound(bundle);builder.Add(bundle,solid);builder.Add(bundle,seed);builder.Add(bundle,first);builder.Add(bundle,second);
  if(!BRepTools::Write(bundle,(std::string(argv[1])+"/input-"+std::to_string(index)+".brep").c_str()))return 1;
  std::cout<<index<<" joined="<<joined<<" radius1="<<r1<<" radius2="<<r2<<" volume="<<props.Mass()<<std::endl;
 }
}
