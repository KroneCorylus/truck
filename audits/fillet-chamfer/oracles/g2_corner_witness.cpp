// Independent G2 radius/section witness. Input coordinates are capture scale10.
#include <BRepAdaptor_Surface.hxx>
#include <BRepAdaptor_Curve.hxx>
#include <BRepClass3d_SolidClassifier.hxx>
#include <BRepClass_FaceClassifier.hxx>
#include <BRepTools.hxx>
#include <BRep_Builder.hxx>
#include <STEPControl_Reader.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <cmath>
#include <iomanip>
#include <iostream>
#include <stdexcept>

// The center is r from x=200, z-5x=500 and the quadratic extrusion.
double station() {
  double lo=0., hi=1.;
  for (int i=0;i<60;++i) {
    const double s=(lo+hi)/2.;
    const double residual=500*(s*s-1)+10*(26/std::sqrt(26+s*s)+std::sqrt(26));
    if (residual<0) lo=s; else hi=s;
  }
  return (lo+hi)/2.;
}
gp_Pnt center() {
  const double s=station();
  return {190.,1000*s-10*s/std::sqrt(26+s*s),1450.-10*std::sqrt(26)};
}
void point(const gp_Pnt& p) { std::cout << p.X() << ' ' << p.Y() << ' ' << p.Z(); }
const char* state(const TopoDS_Shape& shape,const gp_Pnt& p) {
  const auto s=BRepClass3d_SolidClassifier(shape,p,1e-7).State();
  return s==TopAbs_IN?"IN":s==TopAbs_OUT?"OUT":s==TopAbs_ON?"ON":"UNKNOWN";
}
int main(int argc,char**argv) {
  if(argc!=3) {std::cerr<<"usage: g2_corner_witness native.brep truck.step\n";return 2;}
  TopoDS_Shape native;BRep_Builder b;
  if(!BRepTools::Read(native,argv[1],b)) return 2;
  STEPControl_Reader reader;
  if(reader.ReadFile(argv[2])!=IFSelect_RetDone||!reader.TransferRoots()) return 2;
  const auto truck=reader.OneShape(); const auto c=center();
  std::cout<<std::setprecision(17)<<"center ";point(c);std::cout<<'\n';
  gp_Pnt native_witness;
  for (const auto& entry : {std::make_pair("native",native),std::make_pair("truck",truck)}) {
    double edge_worst=0.; gp_Pnt edge_point; int edge_samples=0;
    auto cylinder_residual=[&](const gp_Pnt& p){return std::abs(std::hypot(p.X()-c.X(),p.Z()-c.Z())-10.);};
    for(TopExp_Explorer it(entry.second,TopAbs_EDGE);it.More();it.Next()) {
      const BRepAdaptor_Curve curve(TopoDS::Edge(it.Current()));
      const double a=curve.FirstParameter(),b=curve.LastParameter();
      const auto p=curve.Value(a),q=curve.Value(b);
      if(p.Y()>-800 || q.Y()>-800 || cylinder_residual(p)>1e-4 || cylinder_residual(q)>1e-4) continue;
      for(int i=0;i<=512;++i) {
        const auto x=curve.Value(a+(b-a)*i/512.);const double e=cylinder_residual(x);
        ++edge_samples;if(e>edge_worst){edge_worst=e;edge_point=x;}
      }
    }
    std::cout<<entry.first<<" remote_edge_samples "<<edge_samples<<" cylinder_residual "<<edge_worst<<" point ";point(edge_point);std::cout<<'\n';
    int index=0;
    for(TopExp_Explorer it(entry.second,TopAbs_FACE);it.More();it.Next()) {
      ++index; if(index!=8) continue;
      const auto face=TopoDS::Face(it.Current());const BRepAdaptor_Surface surface(face);
      double u0,u1,v0,v1;BRepTools::UVBounds(face,u0,u1,v0,v1);
      double worst=0,wu=0,wv=0;gp_Pnt wp;int count=0;
      for(int i=0;i<=100;++i) for(int j=0;j<=100;++j) {
        const double u=u0+(u1-u0)*i/100.,v=v0+(v1-v0)*j/100.;
        const auto inside=BRepClass_FaceClassifier(face,gp_Pnt2d(u,v),1e-8).State();
        if(inside!=TopAbs_IN && inside!=TopAbs_ON) continue;
        ++count; const auto p=surface.Value(u,v); const double residual=std::abs(p.Distance(c)-10.);
        if(residual>worst){worst=residual;wp=p;wu=u;wv=v;}
      }
      std::cout<<entry.first<<" face "<<index<<" surface_type "<<surface.GetType()<<" samples "<<count<<" radius_residual "<<worst<<" uv "<<wu<<' '<<wv<<" point ";point(wp);std::cout<<'\n';
      if(entry.first==std::string("native")) native_witness=wp;
    }
  }
  const gp_Vec direction(c,native_witness);const double length=direction.Magnitude();
  const auto unit=direction/length;
  // Radial plane section: sample between the captured boundary and ideal sphere.
  const auto midpoint=c.Translated(unit*((length+10.)/2.));
  std::cout<<"section_midpoint ";point(midpoint);
  std::cout<<" radius "<<midpoint.Distance(c)<<" native "<<state(native,midpoint)<<" truck "<<state(truck,midpoint)<<'\n';
  const double selected_station=station();
  const gp_Vec n(1,0,0), m(-5/std::sqrt(26),0,1/std::sqrt(26));
  const gp_Vec normal(5/std::sqrt(26+selected_station*selected_station),selected_station/std::sqrt(26+selected_station*selected_station),-1/std::sqrt(26+selected_station*selected_station));
  const double det=n.Dot(m.Crossed(normal));
  std::cout<<"section_direction_positive_cone_coefficients "<<unit.Dot(m.Crossed(normal))/det<<' '<<n.Dot(unit.Crossed(normal))/det<<' '<<n.Dot(m.Crossed(unit))/det<<'\n';
  std::cout<<"source_margins "<<midpoint.X()<<' '<<200-midpoint.X()<<' '<<midpoint.Z()-5*midpoint.X()-midpoint.Y()*midpoint.Y()/2000<<' '<<500+5*midpoint.X()-midpoint.Z()<<'\n';
}
