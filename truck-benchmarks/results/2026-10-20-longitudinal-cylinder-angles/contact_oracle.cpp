#include <BRepTools.hxx>
#include <STEPControl_Reader.hxx>
#include <string>
#include <BRepAdaptor_Surface.hxx>
#include <TopExp_Explorer.hxx>
#include <BRep_Tool.hxx>
#include <BRep_Builder.hxx>
#include <TopoDS.hxx>
#include <TopExp.hxx>
#include <TopTools_IndexedMapOfShape.hxx>
#include <iostream>
#include <iomanip>
int main(int argc,char**argv) {
 std::cout<<std::setprecision(17)<<"[";
 for(int i=1;i<argc;++i) {
  TopoDS_Shape shape;
  if(std::string(argv[i]).find(".step")!=std::string::npos) {
   STEPControl_Reader reader;
   if(reader.ReadFile(argv[i])!=IFSelect_RetDone)return 1;
   auto old=std::cout.rdbuf(std::cerr.rdbuf());reader.TransferRoots();std::cout.rdbuf(old);
   shape=reader.OneShape();
  } else {BRep_Builder b;if(!BRepTools::Read(shape,argv[i],b))return 1;}
  TopTools_IndexedMapOfShape vertices;TopExp::MapShapes(shape,TopAbs_VERTEX,vertices);
  if(i>1)std::cout<<",";std::cout<<"{\"vertices\":[";bool first=true;
  for(int j=1;j<=vertices.Extent();++j) {
   auto p=BRep_Tool::Pnt(TopoDS::Vertex(vertices(j)));
   if(std::abs(p.Z())>1e-8)continue;
   if(!first)std::cout<<",";first=false;std::cout<<"["<<p.X()<<","<<p.Y()<<","<<p.Z()<<"]";
  }
  std::cout<<"],\"bevel_planes\":[";first=true;
  for(TopExp_Explorer it(shape,TopAbs_FACE);it.More();it.Next()) {
   BRepAdaptor_Surface surface(TopoDS::Face(it.Current()));
   if(surface.GetType()!=GeomAbs_Plane)continue;
   auto plane=surface.Plane();auto n=plane.Axis().Direction();auto o=plane.Location();
   if(std::abs(n.Z())>1e-8||std::abs(n.X())<1e-8||std::abs(n.Y())<1e-8)continue;
   if(!first)std::cout<<",";first=false;
   std::cout<<"{\"normal\":["<<n.X()<<","<<n.Y()<<","<<n.Z()<<"],\"origin\":["<<o.X()<<","<<o.Y()<<","<<o.Z()<<"]}";
  }
  std::cout<<"]}";
 }
 std::cout<<"]\n";
}
