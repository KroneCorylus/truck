// Independent canonical F9 positive termination: intersection of the original
// parabolic support d=(50-y*y/200)/sqrt(26) with the exact cap cylinder.
#include <STEPControl_Reader.hxx>
#include <BRepTools.hxx>
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <BRepAdaptor_Curve.hxx>
#include <TopExp.hxx>
#include <TopTools_IndexedMapOfShape.hxx>
#include <TopoDS.hxx>
#include <BRepClass3d_SolidClassifier.hxx>
#include <cmath>
#include <iostream>
#include <iomanip>
int main(int argc,char**argv) {
 if(argc!=3)return 2;
 TopoDS_Shape shape;
 if(std::string(argv[1])=="step") {
  STEPControl_Reader r;if(r.ReadFile(argv[2])!=IFSelect_RetDone||!r.TransferRoots())return 3;shape=r.OneShape();
 } else {BRep_Builder b;if(!BRepTools::Read(shape,argv[2],b))return 3;}
 const double k=std::sqrt(26.), U=770/k-5-k;
 auto residual=[&](gp_Pnt p){double x=p.X()/10,y=p.Y()/10,z=p.Z()/10,u=(x+5*z)/k,d=(50-z+5*x)/k;return std::pair<double,double>{std::hypot(u-U,d-1)-1,d-(50-y*y/200)/k};};
 TopTools_IndexedMapOfShape edges;TopExp::MapShapes(shape,TopAbs_EDGE,edges);
 int found=0;
 for(int i=1;i<=edges.Extent();++i){auto e=TopoDS::Edge(edges(i));BRepAdaptor_Curve c(e);double a=c.FirstParameter(),b=c.LastParameter();auto mid=c.Value((a+b)/2);auto er=residual(mid);
 if(mid.Y()<0||std::abs(er.first)>.01||std::abs(er.second)>.01)continue;
 double radial=0,profile=0;gp_Pnt worst;double parameter=0;
 for(int j=0;j<=4096;++j){double t=a+(b-a)*j/4096;auto p=c.Value(t);auto r=residual(p);if(std::abs(r.first)>radial){radial=std::abs(r.first);worst=p;parameter=t;}profile=std::max(profile,std::abs(r.second));}
 auto w=residual(worst);++found;
 std::cout<<std::setprecision(17)<<"{\"edge\":"<<i<<",\"samples\":4097,\"tolerance_capture\":"<<BRep_Tool::Tolerance(e)<<",\"max_cylinder_radial_capture\":"<<10*radial<<",\"max_profile_depth_capture\":"<<10*profile<<",\"worst_parameter\":"<<parameter<<",\"worst_xyz_capture\":["<<worst.X()<<","<<worst.Y()<<","<<worst.Z()<<"],\"worst_profile_residual_capture\":"<<10*w.second<<"}\n";
 }
 for(double eps:{.01,.001,.0001}) {
  int failures=0,total=0,on=0,wrong_side=0,unknown=0;
  for(int j=1;j<100;++j) {
   double d=(1+5/k)*j/100, q=std::sqrt(2*d-d*d),u=U+q,y=std::sqrt(10000-200*k*d);
   double py=y/(100*k),pn=std::hypot(py,1),nu=q,ny=py/pn,nd=d-1+1/pn,n=std::sqrt(nu*nu+ny*ny+nd*nd);
   for(int sign:{-1,1}) {
    double uu=u+sign*eps/10*nu/n,yy=y+sign*eps/10*ny/n,dd=d+sign*eps/10*nd/n;
    gp_Pnt p(10*((uu+5*dd)/k-250./26),10*yy,10*((5*uu-dd)/k+50./26));
    BRepClass3d_SolidClassifier cl(shape,p,1e-7);++total;
    if(cl.State()!=(sign<0?TopAbs_IN:TopAbs_OUT))++failures;
    if(cl.State()==TopAbs_ON)++on;
    else if(cl.State()==TopAbs_UNKNOWN)++unknown;
    else if(cl.State()!=(sign<0?TopAbs_IN:TopAbs_OUT))++wrong_side;
   }
  }
  std::cout<<"{\"termination_material_offset_capture\":"<<eps<<",\"samples\":"<<total<<",\"failures\":"<<failures<<",\"on\":"<<on<<",\"unknown\":"<<unknown<<",\"wrong_side\":"<<wrong_side<<"}\n";
 }
 return found?0:4;
}
