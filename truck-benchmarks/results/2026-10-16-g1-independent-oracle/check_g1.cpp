#include <BRepTools.hxx>
#include <BRep_Builder.hxx>
#include <BRepAdaptor_Surface.hxx>
#include <BRepAdaptor_Curve.hxx>
#include <BRepClass_FaceClassifier.hxx>
#include <STEPControl_Reader.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <BRepFilletAPI_MakeFillet.hxx>
#include <BRepBuilderAPI_Copy.hxx>
#include <sstream>
#include <BRepCheck_Analyzer.hxx>
#include <BRepGProp.hxx>
#include <GProp_GProps.hxx>
#include <array>
#include <stdexcept>
#include <cmath>
#include <algorithm>
#include <iostream>
#include <iomanip>
gp_Pnt center(double q) {double g=std::sqrt(26+q*q/1e6);return gp_Pnt(-800,q*(1-1/g),q*q/2000-4000+26000/g);}
double closest_parameter(const gp_Pnt& p) {
 // C'(q)=A(q)*(0,1,q/1000), with A>=1-1/sqrt(26).
 // For z<=1500, the stationary equation is strictly increasing on the real line.
 if(p.Z()>1500.000001)throw std::runtime_error("outside proven minimization domain");
 auto h=[&](double q){auto c=center(q);return c.Y()-p.Y()+q/1000*(c.Z()-p.Z());};
 double a=-2000,b=2000;
 if(h(a)>=0||h(b)<=0)throw std::runtime_error("unbracketed center projection");
 for(int i=0;i<100;++i){double q=(a+b)/2;if(h(q)<0)a=q;else b=q;}
 return (a+b)/2;
}
double plane_error(const gp_Pnt& p) {
 double q=p.Y()*1.24;
 for(int i=0;i<20;++i) {double g=std::sqrt(26+q*q/1e6);q-=(q*(1-1/g)-p.Y())/(1-1/g+q*q/(1e6*g*g*g));}
 return std::abs(p.Z()-center(q).Z());
}
void inspect(const char*label,const TopoDS_Shape&s,int grid) {
 double worst=0,contact=0,core=0,extension=0,plane_distance=0,qmax=0,worstq=0;gp_Pnt where;int samples=0,contacts=0,faces=0,splines=0;
 for(TopExp_Explorer it(s,TopAbs_FACE);it.More();it.Next()) {
  auto face=TopoDS::Face(it.Current());BRepAdaptor_Surface surface(face);
  ++faces; if(surface.GetType()!=GeomAbs_BSplineSurface)continue; ++splines;
  double u0,u1,v0,v1;BRepTools::UVBounds(face,u0,u1,v0,v1);
  auto check=[&](const gp_Pnt&p){double q=closest_parameter(p),e=std::abs(p.Distance(center(q))-1000);qmax=std::max(qmax,std::abs(q));
   (std::abs(q)<=1000?core:extension)=std::max(std::abs(q)<=1000?core:extension,e);
   if(e>worst){worst=e;where=p;worstq=q;}++samples;
   if(std::abs(p.X()-200)<1e-6){contact=std::max(contact,plane_error(p));auto c=center(q);c.SetX(200);plane_distance=std::max(plane_distance,p.Distance(c));++contacts;}};
  for(int i=0;i<=grid;++i)for(int j=0;j<=grid;++j){double u=u0+(u1-u0)*i/grid,v=v0+(v1-v0)*j/grid;auto state=BRepClass_FaceClassifier(face,gp_Pnt2d(u,v),1e-8).State();if(state==TopAbs_IN||state==TopAbs_ON)check(surface.Value(u,v));}
  for(TopExp_Explorer e(face,TopAbs_EDGE);e.More();e.Next()){BRepAdaptor_Curve c(TopoDS::Edge(e.Current()));for(int i=0;i<=2*grid;++i)check(c.Value(c.FirstParameter()+(c.LastParameter()-c.FirstParameter())*i/(2*grid)));}
 }
 if(faces!=5||splines!=1||contacts<grid||samples<grid*grid/2)throw std::runtime_error("unexpected output sampling scope");
 std::cout<<label<<" sampled_radius_residual="<<worst<<" location="<<where.X()<<","<<where.Y()<<","<<where.Z()<<" faces="<<faces<<" splines="<<splines<<" samples="<<samples<<" plane_contact_residual="<<contact<<" contacts="<<contacts<<" plane_contact_distance="<<plane_distance<<" core_residual="<<core<<" extension_residual="<<extension<<" qmax="<<qmax<<" worstq="<<worstq<<std::endl;
}
void verify_input(const TopoDS_Shape& s) {
 std::array<int,4> supports{};int count=0;
 for(TopExp_Explorer it(s,TopAbs_FACE);it.More();it.Next()) {
  auto face=TopoDS::Face(it.Current());BRepAdaptor_Surface surface(face);
  double u0,u1,v0,v1;BRepTools::UVBounds(face,u0,u1,v0,v1);
  if(surface.GetType()==GeomAbs_SurfaceOfExtrusion){
   auto basis=surface.BasisCurve();if(basis->GetType()!=GeomAbs_Parabola)throw std::runtime_error("nonparabolic input");
   auto parabola=basis->Parabola();auto location=parabola.Location();auto x=parabola.XAxis().Direction();auto y=parabola.YAxis().Direction();auto d=surface.Direction();
   std::cout<<"parabola focal="<<parabola.Focal()<<" origin="<<location.X()<<","<<location.Y()<<","<<location.Z()<<" xdir="<<x.X()<<","<<x.Y()<<","<<x.Z()<<" ydir="<<y.X()<<","<<y.Y()<<","<<y.Z()<<" extrusion="<<d.X()<<","<<d.Y()<<","<<d.Z()<<std::endl;
   if(std::abs(parabola.Focal()-500)>1e-9||location.Distance(gp_Pnt(0,0,0))>1e-9||x.Dot(gp_Dir(0,0,1))<1-1e-12||std::abs(y.Dot(gp_Dir(0,1,0)))<1-1e-12||std::abs(d.Dot(gp_Dir(1,0,5)))<1-1e-12)throw std::runtime_error("unexpected parabola/extrusion parameters");
  }
  std::array<double,4> residual{};
  for(int i=0;i<=32;++i)for(int j=0;j<=32;++j){auto p=surface.Value(u0+(u1-u0)*i/32.,v0+(v1-v0)*j/32.);
   std::array<double,4> r={p.X(),p.X()-200,p.Z()-p.Y()*p.Y()/2000-5*p.X(),p.Z()-5*p.X()-500};
   for(int k=0;k<4;++k)residual[k]=std::max(residual[k],std::abs(r[k]));}
  int which=std::min_element(residual.begin(),residual.end())-residual.begin();
  std::cout<<"input face="<<++count<<" type="<<surface.GetType()<<" support="<<which<<" residual="<<residual[which]<<std::endl;
  if(residual[which]>1e-9)throw std::runtime_error("wrong analytic support");
  ++supports[which];
 }
 if(supports!=std::array<int,4>{1,1,1,1})throw std::runtime_error("unexpected input topology");
 GProp_GProps props;BRepGProp::VolumeProperties(s,props);
 if(std::abs(props.Mass()-400000000./3)>1e-5)throw std::runtime_error("input volume mismatch");
 std::cout<<"input volume="<<props.Mass()<<" analytic="<<400000000./3<<std::endl;
}
void controls(){
 double exact=0,shift_error=0,plane=0;int samples=0;
 for(int i=-120;i<=120;++i){double q=i*10.,m=std::sqrt(1+q*q/1e6);auto c=center(q);
  for(int j=0;j<=20;++j){double angle=std::atan(m/5)*j/20.;gp_Vec n(std::cos(angle),q/(1000*m)*std::sin(angle),-std::sin(angle)/m);
   auto p=c.Translated(n*1000),shifted=c.Translated(n*1000.01);
   if(p.Z()>1500||shifted.Z()>1500)continue;
   exact=std::max(exact,std::abs(p.Distance(center(closest_parameter(p)))-1000));
   shift_error=std::max(shift_error,std::abs(std::abs(shifted.Distance(center(closest_parameter(shifted)))-1000)-.01));
   if(j==0)plane=std::max(plane,plane_error(p));
   ++samples;
  }
 }
 if(samples<4000||exact>1e-9||shift_error>1e-9||plane>1e-9)throw std::runtime_error("analytic controls failed");
 bool domain_rejected=false,bracket_rejected=false;
 try{closest_parameter(gp_Pnt(200,0,2000));}catch(const std::runtime_error&){domain_rejected=true;}
 try{closest_parameter(gp_Pnt(200,1e9,1000));}catch(const std::runtime_error&){bracket_rejected=true;}
 if(!domain_rejected||!bracket_rejected)throw std::runtime_error("invalid projection accepted");
 std::cout<<"controls samples="<<samples<<" exact_residual="<<exact<<" shifted_radius_error="<<shift_error<<" exact_plane_error="<<plane<<" invalid_domain_and_bracket_rejected=1"<<std::endl;
}
int main(int argc,char**argv) {
 if(argc!=5)return 1;
 int grid=std::stoi(argv[4]);if(grid<8||grid>512)return 1;
 std::cout<<std::setprecision(17);
 BRep_Builder b;TopoDS_Shape input,native;
 if(!BRepTools::Read(input,argv[1],b)||!BRepTools::Read(native,argv[2],b))return 2;
 verify_input(input);controls();
 std::ostringstream before;BRepTools::Write(input,before);
 STEPControl_Reader r;if(r.ReadFile(argv[3])!=IFSelect_RetDone||!r.TransferRoots())return 3;
 inspect("native",native,grid);inspect("Truck",r.OneShape(),grid);
 TopoDS_Edge seed;int seeds=0;
 for(TopExp_Explorer it(input,TopAbs_EDGE);it.More();it.Next()){
  BRepAdaptor_Curve c(TopoDS::Edge(it.Current()));
  if(c.GetType()==GeomAbs_Parabola&&std::abs(c.Value((c.FirstParameter()+c.LastParameter())/2).X()-200)<1e-9){if(seed.IsNull()){seed=TopoDS::Edge(it.Current());++seeds;}else if(!seed.IsSame(it.Current()))++seeds;}
 }
 if(seeds!=1)throw std::runtime_error("ambiguous seed");
 for(bool tight:{false,true}){
  BRepBuilderAPI_Copy copy(input);
  BRepFilletAPI_MakeFillet builder(copy.Shape());
  if(tight)builder.SetParams(1e-6,1e-8,1e-8,1e-8,1e-8,1e-8);
  else builder.SetParams(.01,1e-4,1e-5,1e-4,1e-5,.001);
  builder.SetContinuity(GeomAbs_C1,tight?1e-6:.01);builder.Add(1000,TopoDS::Edge(copy.ModifiedShape(seed)));builder.Build();
  if(!builder.IsDone()){std::cout<<"rebuilt tight="<<tight<<" failed"<<std::endl;continue;}
  std::cout<<"rebuilt tight="<<tight<<" valid="<<BRepCheck_Analyzer(builder.Shape()).IsValid()<<std::endl;
  inspect(tight?"native-tight":"native-default",builder.Shape(),grid);
 }
 std::ostringstream after;BRepTools::Write(input,after);
 if(before.str()!=after.str())throw std::runtime_error("input mutation");
 std::cout<<"input_unchanged=1"<<std::endl;
}
