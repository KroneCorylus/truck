#include <BRepAdaptor_Curve.hxx>
#include <BRepAdaptor_Surface.hxx>
#include <BRepPrimAPI_MakeCone.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepGProp.hxx>
#include <BRepTools.hxx>
#include <BRep_Builder.hxx>
#include <GProp_GProps.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Compound.hxx>
#include <gp_Cone.hxx>
#include <cmath>
#include <iostream>
#include <iomanip>
#include <stdexcept>
#include <string>
int main(int argc,char**argv) {
 if(argc!=2)return 1;
 std::cout<<std::setprecision(17);
 for(int k=0;k<2;++k){
  const double bottom=k==0?5.:3.,top=k==0?3.:5.,height=10.;
  TopoDS_Shape solid=BRepPrimAPI_MakeCone(bottom,top,height);
  TopoDS_Face cap,cone;TopoDS_Edge seed;
  for(TopExp_Explorer it(solid,TopAbs_FACE);it.More();it.Next()){
   auto face=TopoDS::Face(it.Current());BRepAdaptor_Surface s(face);
   if(s.GetType()==GeomAbs_Cone)cone=face;
   if(s.GetType()==GeomAbs_Plane&&std::abs(s.Plane().Location().Z()-height)<1e-10)cap=face;
  }
  if(cap.IsNull()||cone.IsNull()||!BRepCheck_Analyzer(solid).IsValid())throw std::runtime_error("invalid frustum");
  for(TopExp_Explorer it(cap,TopAbs_EDGE);it.More();it.Next()){
   auto edge=TopoDS::Edge(it.Current());if(BRepAdaptor_Curve(edge).GetType()==GeomAbs_Circle)seed=edge;
  }
  if(seed.IsNull())throw std::runtime_error("missing rim");
  GProp_GProps props;BRepGProp::VolumeProperties(solid,props);
  const double expected=std::acos(-1.)*height/3*(bottom*bottom+bottom*top+top*top);
  if(std::abs(props.Mass()-expected)>1e-9)throw std::runtime_error("input material mismatch");
  TopoDS_Compound bundle;BRep_Builder b;b.MakeCompound(bundle);b.Add(bundle,solid);b.Add(bundle,seed);b.Add(bundle,cap);b.Add(bundle,cone);
  const std::string path=std::string(argv[1])+"/input-"+std::to_string(k)+".brep";
  if(!BRepTools::Write(bundle,path.c_str()))throw std::runtime_error("write failed");
  const auto c=BRepAdaptor_Surface(cone).Cone();auto p=c.Apex();auto a=c.Axis().Direction();
  std::cout<<k<<" volume="<<props.Mass()<<" semiangle="<<c.SemiAngle()<<" apex="<<p.X()<<","<<p.Y()<<","<<p.Z()<<" axis="<<a.X()<<","<<a.Y()<<","<<a.Z()<<std::endl;
 }
}
