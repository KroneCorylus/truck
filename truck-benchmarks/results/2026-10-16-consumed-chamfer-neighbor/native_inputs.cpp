#include <BRepAdaptor_Curve.hxx>
#include <BRepAdaptor_Surface.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepFilletAPI_MakeChamfer.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepGProp.hxx>
#include <BRepTools.hxx>
#include <BRep_Builder.hxx>
#include <GProp_GProps.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Compound.hxx>
#include <cmath>
#include <iostream>
#include <iomanip>
#include <stdexcept>
#include <string>
int main(int argc,char**argv){
 if(argc!=2)return 1;
 std::cout<<std::setprecision(17);
 for(int k=0;k<3;++k){
  const double scale=k==2?.2:1.,size=10*scale;
  TopoDS_Shape block=BRepPrimAPI_MakeBox(size,size,size);TopoDS_Edge corner;
  for(TopExp_Explorer it(block,TopAbs_EDGE);it.More();it.Next()){
   auto edge=TopoDS::Edge(it.Current());BRepAdaptor_Curve c(edge);auto p=c.Value((c.FirstParameter()+c.LastParameter())/2);
   if(std::abs(p.X()-size)<1e-10&&std::abs(p.Y()-size)<1e-10&&std::abs(p.Z()-size/2)<1e-10)corner=edge;
  }
  if(corner.IsNull())throw std::runtime_error("missing first edge");
  BRepFilletAPI_MakeChamfer first(block);first.Add(scale,corner);first.Build();
  if(!first.IsDone())throw std::runtime_error("first chamfer failed");
  auto input=first.Shape();TopoDS_Face cap;TopoDS_Edge seed;int faces=0;
  for(TopExp_Explorer it(input,TopAbs_FACE);it.More();it.Next()){
   ++faces;auto face=TopoDS::Face(it.Current());BRepAdaptor_Surface s(face);
   if(s.GetType()==GeomAbs_Plane&&std::abs(s.Plane().Location().Z())<1e-10&&std::abs(s.Plane().Axis().Direction().Z())>.99)cap=face;
  }
  if(cap.IsNull()||faces!=7||!BRepCheck_Analyzer(input).IsValid())throw std::runtime_error("invalid beveled box");
  for(TopExp_Explorer it(cap,TopAbs_EDGE);it.More();it.Next()){
   auto edge=TopoDS::Edge(it.Current());BRepAdaptor_Curve c(edge);auto p=c.Value((c.FirstParameter()+c.LastParameter())/2);
   if(std::abs(p.Y()-size)<1e-10&&std::abs(p.Z())<1e-10)seed=edge;
  }
  if(seed.IsNull())throw std::runtime_error("missing second edge");
  GProp_GProps props;BRepGProp::VolumeProperties(input,props);
  if(std::abs(props.Mass()-995*scale*scale*scale)>1e-9)throw std::runtime_error("input material mismatch");
  TopoDS_Compound bundle;BRep_Builder b;b.MakeCompound(bundle);b.Add(bundle,input);b.Add(bundle,seed);b.Add(bundle,cap);
  const auto path=std::string(argv[1])+"/input-"+std::to_string(k)+".brep";
  if(!BRepTools::Write(bundle,path.c_str()))throw std::runtime_error("write failed");
  std::cout<<k<<" scale="<<scale<<" faces="<<faces<<" volume="<<props.Mass()<<std::endl;
 }
}
