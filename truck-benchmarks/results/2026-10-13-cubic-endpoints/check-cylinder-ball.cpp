#include <STEPControl_Reader.hxx>
#include <BRepTools.hxx>
#include <BRep_Builder.hxx>
#include <BRepAdaptor_Surface.hxx>
#include <BRepClass_FaceClassifier.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <algorithm>
#include <cmath>
#include <iostream>
#include <string>

int main(int argc,char**argv) {
  if(argc!=3)return 2;
  const std::string name(argv[2]);
  if(name!="K2" && name!="K3" && name!="K4")return 2;
  const bool tilted=name=="K4";
  // A radius-5 ball inside the outer cylinder and outside the cut cylinder
  // has its center on their inward/outward offset cylinders, respectively.
  const double outer_radius=tilted ? 25. : 45.;
  const double hole_radius=tilted ? 45. : 35.;
  const gp_Pnt hole_origin=tilted ? gp_Pnt(50.,140.,-30.) : gp_Pnt(50.,120.,70.);
  const double angle=80.*std::acos(-1.)/180.;
  const gp_Vec hole_axis=tilted ? gp_Vec(0.,-std::sin(angle),std::cos(angle)) : gp_Vec(0.,-1.,0.);
  TopoDS_Shape shape;
  if(std::string(argv[1]).ends_with(".step")) {
    STEPControl_Reader r;
    if(r.ReadFile(argv[1])!=IFSelect_RetDone || !r.TransferRoots())return 3;
    shape=r.OneShape();
  } else {
    BRep_Builder b;
    if(!BRepTools::Read(shape,argv[1],b))return 3;
  }
  std::cout.precision(17);
  int face_index=0;
  for(TopExp_Explorer ex(shape,TopAbs_FACE);ex.More();ex.Next(),++face_index) {
    auto face=TopoDS::Face(ex.Current());
    BRepAdaptor_Surface surface(face);
    if(surface.GetType()!=GeomAbs_BSplineSurface)continue;
    double u0,u1,v0,v1;BRepTools::UVBounds(face,u0,u1,v0,v1);
    double max_error=0.;gp_Pnt worst;int samples=0;
    for(int i=1;i<64;++i)for(int j=1;j<64;++j) {
      const double u=u0+(u1-u0)*i/64.,v=v0+(v1-v0)*j/64.;
      if(BRepClass_FaceClassifier(face,gp_Pnt2d(u,v),1e-8).State()!=TopAbs_IN)continue;
      gp_Pnt p;gp_Vec du,dv;surface.D1(u,v,p,du,dv);
      auto n=du.Crossed(dv);
      if(n.Magnitude()<1e-12)continue;
      n.Normalize();double error=1e100;
      for(double sign:{-1.,1.}) {
        auto center=p.Translated(n*(sign*5.));
        auto outer=std::abs(std::hypot(center.X()-50.,center.Y()-50.)-outer_radius);
        auto radial=gp_Vec(hole_origin,center);
        auto inner=std::abs(radial.Crossed(hole_axis).Magnitude()-hole_radius);
        error=std::min(error,std::max(outer,inner));
      }
      if(error>max_error){max_error=error;worst=p;}
      ++samples;
    }
    std::cout<<"face "<<face_index<<" samples "<<samples<<" max_center_offset_error "<<max_error<<" at "<<worst.X()<<','<<worst.Y()<<','<<worst.Z()<<'\n';
  }
}
