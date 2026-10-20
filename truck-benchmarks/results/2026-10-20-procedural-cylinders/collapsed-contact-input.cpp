#include <BRepAdaptor_Curve.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepFilletAPI_MakeFillet.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepTools.hxx>
#include <BRep_Builder.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Compound.hxx>
#include <cmath>
#include <vector>
#include <stdexcept>
int main(int argc,char**argv){
 if(argc!=4 && argc!=5)return 1;bool closed=argc==5;double size=std::stod(argv[2]),radius=std::stod(argv[3]);
 auto cube=BRepPrimAPI_MakeBox(size,size,size).Shape();TopoDS_Edge edge;
 for(TopExp_Explorer it(cube,TopAbs_EDGE);it.More();it.Next()){
  auto e=TopoDS::Edge(it.Current());BRepAdaptor_Curve c(e);auto p=c.Value((c.FirstParameter()+c.LastParameter())/2);
  if(std::abs(p.X())<1e-8 && std::abs(p.Y())<1e-8)edge=e;
 }
 if(edge.IsNull())throw std::runtime_error("missing first edge");
 BRepFilletAPI_MakeFillet first(cube);
 if(closed){std::vector<TopoDS_Edge> added;for(TopExp_Explorer it(cube,TopAbs_EDGE);it.More();it.Next()){
 auto e=TopoDS::Edge(it.Current());BRepAdaptor_Curve c(e);auto a=c.Value(c.FirstParameter()),b=c.Value(c.LastParameter());
 bool seen=false;for(auto old:added)if(old.IsSame(e))seen=true;
 if(!seen && std::abs(a.Z()-b.Z())>size/2){first.Add(radius,e);added.push_back(e);}
 }}else first.Add(radius,edge);first.Build();if(!first.IsDone())throw std::runtime_error("first fillet failed");
 auto solid=first.Shape();if(!BRepCheck_Analyzer(solid).IsValid())throw std::runtime_error("invalid input");TopoDS_Edge seed;
 for(TopExp_Explorer it(solid,TopAbs_EDGE);it.More();it.Next()){
  auto e=TopoDS::Edge(it.Current());BRepAdaptor_Curve c(e);auto p=c.Value((c.FirstParameter()+c.LastParameter())/2);
  if(c.GetType()==GeomAbs_Line && std::abs(p.Y())<1e-8 && std::abs(p.Z()-size)<1e-8)seed=e;
 }
 if(seed.IsNull())throw std::runtime_error("missing second edge");
 TopoDS_Compound bundle;BRep_Builder b;b.MakeCompound(bundle);b.Add(bundle,solid);b.Add(bundle,seed);
 return BRepTools::Write(bundle,argv[1])?0:1;
}
