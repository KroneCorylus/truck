#include <BRepAdaptor_Curve.hxx>
#include <BRepAdaptor_Surface.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepPrimAPI_MakeRevol.hxx>
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
int main(int argc,char**argv) {
 if(argc!=2)return 1;
 std::cout<<std::setprecision(17);
 int index=0;
 for(double scale:{1.,.2}) {
  BRepBuilderAPI_MakePolygon polygon;
  for(auto p:{gp_Pnt(0,.25,0),gp_Pnt(1.2,.25,0),gp_Pnt(1.2,.95,0),gp_Pnt(1,1,0),gp_Pnt(0,1,0)}) {
   p.SetXYZ(p.XYZ()*scale);polygon.Add(p);
  }
  polygon.Close();
  TopoDS_Shape solid=BRepPrimAPI_MakeRevol(BRepBuilderAPI_MakeFace(polygon.Wire()).Face(),gp_Ax1(gp_Pnt(0,0,0),gp_Dir(1,0,0)),std::acos(-1.)/2).Shape();
  TopoDS_Face plane;TopoDS_Edge seed;int faces=0;
  for(TopExp_Explorer it(solid,TopAbs_FACE);it.More();it.Next()) {
   ++faces;auto f=TopoDS::Face(it.Current());BRepAdaptor_Surface surface(f);
   if(surface.GetType()==GeomAbs_Plane&&std::abs(surface.Plane().Location().Z())<1e-10&&std::abs(surface.Plane().Axis().Direction().Z())>.99)plane=f;
  }
  if(plane.IsNull()||!BRepCheck_Analyzer(solid).IsValid())throw std::runtime_error("invalid input");
  for(TopExp_Explorer it(plane,TopAbs_EDGE);it.More();it.Next()) {
   auto e=TopoDS::Edge(it.Current());BRepAdaptor_Curve c(e);auto p=c.Value((c.FirstParameter()+c.LastParameter())/2);
   if(c.GetType()==GeomAbs_Line&&std::abs(p.Y()-scale)<1e-10&&std::abs(p.X()-.5*scale)<1e-10)seed=e;
  }
  if(seed.IsNull())throw std::runtime_error("missing seed");
  GProp_GProps props;BRepGProp::VolumeProperties(solid,props);
  const double expected=.87584985188205443994*std::pow(scale,3);
  if(std::abs(props.Mass()-expected)>1e-9)throw std::runtime_error("input material mismatch");
  TopoDS_Compound bundle;BRep_Builder b;b.MakeCompound(bundle);b.Add(bundle,solid);b.Add(bundle,seed);b.Add(bundle,plane);
  if(!BRepTools::Write(bundle,(std::string(argv[1])+"/input-"+std::to_string(index)+".brep").c_str()))throw std::runtime_error("write failed");
  std::cout<<index++<<" scale="<<scale<<" faces="<<faces<<" volume="<<props.Mass()<<" expected="<<expected<<std::endl;
 }
}
