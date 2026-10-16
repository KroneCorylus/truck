#include <BRepAdaptor_Curve.hxx>
#include <BRepAdaptor_Surface.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRepPrimAPI_MakePrism.hxx>
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
  for(int k=0;k<3;++k) {
    double scale=k==2?.1:1.,r=5*scale,h=10*scale;
    const gp_Circ circle(gp_Ax2(gp_Pnt(0,0,0),gp_Dir(0,0,1)),r);
    TopoDS_Edge arc=BRepBuilderAPI_MakeEdge(circle,0,std::acos(-1.));
    TopoDS_Edge chord=BRepBuilderAPI_MakeEdge(TopExp::LastVertex(arc),TopExp::FirstVertex(arc));
    TopoDS_Wire wire=BRepBuilderAPI_MakeWire(arc,chord);
    TopoDS_Face base=BRepBuilderAPI_MakeFace(wire);
    TopoDS_Shape solid=BRepPrimAPI_MakePrism(base,gp_Vec(0,0,h));
    TopoDS_Face cap,cylinder;TopoDS_Edge seed;int faces=0;
    for(TopExp_Explorer it(solid,TopAbs_FACE);it.More();it.Next()) {
      ++faces;auto face=TopoDS::Face(it.Current());BRepAdaptor_Surface s(face);
      if(s.GetType()==GeomAbs_Cylinder)cylinder=face;
      if(s.GetType()==GeomAbs_Plane&&std::abs(s.Plane().Location().Y())<1e-10
         &&std::abs(s.Plane().Axis().Direction().Y())>.99)cap=face;
    }
    if(cylinder.IsNull()||cap.IsNull()||faces!=4||!BRepCheck_Analyzer(solid).IsValid())throw std::runtime_error("invalid half-cylinder");
    int edges=0;
    for(TopExp_Explorer it(cap,TopAbs_EDGE);it.More();it.Next()) {
      ++edges;auto e=TopoDS::Edge(it.Current());BRepAdaptor_Curve c(e);
      auto p=c.Value((c.FirstParameter()+c.LastParameter())/2);
      if(c.GetType()==GeomAbs_Line&&std::abs(p.X()-r)<1e-10&&std::abs(p.Z()-h/2)<1e-10)seed=e;
    }
    if(edges!=4||seed.IsNull())throw std::runtime_error("side is not rectangular");
    GProp_GProps props;BRepGProp::VolumeProperties(solid,props);
    double expected=std::acos(-1.)*r*r*h/2;
    if(std::abs(props.Mass()-expected)>1e-9)throw std::runtime_error("input material mismatch");
    TopoDS_Compound bundle;BRep_Builder builder;builder.MakeCompound(bundle);
    builder.Add(bundle,solid);builder.Add(bundle,seed);builder.Add(bundle,cap);builder.Add(bundle,cylinder);
    std::string path=std::string(argv[1])+"/input-"+std::to_string(k)+".brep";
    if(!BRepTools::Write(bundle,path.c_str()))throw std::runtime_error("write failed");
    std::cout<<"input="<<k<<" scale="<<scale<<" faces="<<faces<<" cap_edges="<<edges<<" volume="<<props.Mass()<<" expected="<<expected<<std::endl;
  }
}
