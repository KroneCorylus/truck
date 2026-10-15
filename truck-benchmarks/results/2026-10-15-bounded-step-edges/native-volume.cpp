#include <BRepTools.hxx>
#include <BRep_Builder.hxx>
#include <BRepGProp.hxx>
#include <STEPControl_Reader.hxx>
#include <GProp_GProps.hxx>
#include <iostream>
#include <iomanip>
int main(int argc,char** argv) {
 if(argc!=2)return 2;
 TopoDS_Shape shape;
 std::string path=argv[1];
 if(path.size()>=5 && path.substr(path.size()-5)==".step") {STEPControl_Reader reader;if(reader.ReadFile(argv[1])!=IFSelect_RetDone || !reader.TransferRoots())return 3;shape=reader.OneShape();}
 else {BRep_Builder builder;if(!BRepTools::Read(shape,argv[1],builder))return 4;}
 std::cout<<std::setprecision(17);
 for(double eps:{1e-6,1e-8,1e-10,1e-12}) {
  GProp_GProps normal,gk;BRepGProp::VolumeProperties(shape,normal,eps,true,false);
  auto error=BRepGProp::VolumePropertiesGK(shape,gk,eps,true,true);
  std::cout<<"epsilon "<<eps<<" regular "<<normal.Mass()<<" GK "<<gk.Mass()<<" error "<<error<<"\n";
 }
}
