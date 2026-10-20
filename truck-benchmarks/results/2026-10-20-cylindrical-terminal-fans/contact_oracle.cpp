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
   if(!first)std::cout<<",";first=false;std::cout<<"["<<p.X()<<","<<p.Y()<<","<<p.Z()<<"]";
  }
  std::cout<<"],\"bevel_planes\":[";first=true;
  for(TopExp_Explorer it(shape,TopAbs_FACE);it.More();it.Next()) {
   BRepAdaptor_Surface surface(TopoDS::Face(it.Current()));
   if(surface.GetType()!=GeomAbs_Plane)continue;
   auto plane=surface.Plane();auto n=plane.Axis().Direction();auto o=plane.Location();
   if(!first)std::cout<<",";first=false;
   std::cout<<"{\"normal\":["<<n.X()<<","<<n.Y()<<","<<n.Z()<<"],\"origin\":["<<o.X()<<","<<o.Y()<<","<<o.Z()<<"]}";
  }
  std::cout<<"],\"faces\":[";first=true;
  for(TopExp_Explorer it(shape,TopAbs_FACE);it.More();it.Next()) {
   auto face=TopoDS::Face(it.Current());BRepAdaptor_Surface surface(face);
   if(!first)std::cout<<",";first=false;
   std::cout<<"{\"type\":"<<surface.GetType()<<",\"vertices\":[";
   TopTools_IndexedMapOfShape corners;TopExp::MapShapes(face,TopAbs_VERTEX,corners);
   for(int j=1;j<=corners.Extent();++j) {auto p=BRep_Tool::Pnt(TopoDS::Vertex(corners(j)));if(j>1)std::cout<<",";std::cout<<"["<<p.X()<<","<<p.Y()<<","<<p.Z()<<"]";}
   std::cout<<"],\"samples\":[";
   double u0,u1,v0,v1;BRepTools::UVBounds(face,u0,u1,v0,v1);
   for(int j=0;j<9;++j)for(int k=0;k<9;++k) {auto p=surface.Value(u0+(u1-u0)*j/8.,v0+(v1-v0)*k/8.);if(j||k)std::cout<<",";std::cout<<"["<<p.X()<<","<<p.Y()<<","<<p.Z()<<"]";}
   std::cout<<"]}";
  }
  std::cout<<"]}";
 }
 std::cout<<"]\n";
}
