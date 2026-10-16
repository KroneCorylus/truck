#include <BRepTools.hxx>
#include <BRep_Builder.hxx>
#include <BRepGProp.hxx>
#include <GProp_GProps.hxx>
#include <iostream>
#include <iomanip>
int main(int argc,char**argv) {
 std::cout<<std::setprecision(17);
 for(int i=1;i<argc;++i) {
  TopoDS_Shape shape;BRep_Builder b;if(!BRepTools::Read(shape,argv[i],b))return 1;
  GProp_GProps props;double error=BRepGProp::VolumeProperties(shape,props,1e-12);
  std::cout<<props.Mass()<<" "<<error<<"\n";
 }
}
