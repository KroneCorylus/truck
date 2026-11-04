// H1 retained-artifact witness. OCCT supplies queries only; geometry below is polynomial-derived.
// N = normalize(A cross B'), C = B + A*(10-r+r*Nz)/10 - r*N.
// The canal is C+r*(Z*cos(phi)+V*sin(phi)); the right sphere uses C(ts),
// where M dot C(ts)=-r. Its normals lie in the positive cone of Z,M,N(ts).
// The ruling cylinder translates that sphere's N-to-M arc along A down to z=0.
// Trimming P-Q equates curved/straight canal offsets; Q-R intersects M dot X=0.
// All equations use the H1 polynomial, never fitted Truck centers. Finite samples
// and local refinement are not global bounds; FP arithmetic is not outward rounded.
#include <BRepAdaptor_Surface.hxx>
#include <BRepAdaptor_Curve.hxx>
#include <BRepClass3d_SolidClassifier.hxx>
#include <BRepClass_FaceClassifier.hxx>
#include <BRepExtrema_DistShapeShape.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <BRepBuilderAPI_Copy.hxx>
#include <algorithm>
#include <BRepFilletAPI_MakeFillet.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepTools.hxx>
#include <BRep_Builder.hxx>
#include <STEPControl_Reader.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopTools_IndexedMapOfShape.hxx>
#include <TopoDS.hxx>
#include <cmath>
#include <iomanip>
#include <iostream>
#include <vector>
#include <string>
#include <stdexcept>
#include <BRep_Tool.hxx>
#include <GeomAPI_ProjectPointOnSurf.hxx>
#include <sstream>
#include <functional>
void point(const gp_Pnt&p){std::cout<<'['<<p.X()<<','<<p.Y()<<','<<p.Z()<<']';}
const char* state(const TopoDS_Shape&s,const gp_Pnt&p){auto t=BRepClass3d_SolidClassifier(s,p,1e-7).State();return t==TopAbs_IN?"IN":t==TopAbs_OUT?"OUT":t==TopAbs_ON?"ON":"UNKNOWN";}
struct Near{double d=1e100;gp_Pnt p;int face=0; std::string detail;};
Near nearest(const gp_Pnt&p,const TopoDS_Shape& target,const std::vector<TopoDS_Face>&faces){
 TopoDS_Compound boundary; BRep_Builder builder; builder.MakeCompound(boundary);
 const auto copy=BRepBuilderAPI_Copy(target,false,false).Shape();
 for(TopExp_Explorer it(copy,TopAbs_FACE);it.More();it.Next())builder.Add(boundary,it.Current());
 // Query boundary faces, never the solid interior (which would report zero).
 BRepExtrema_DistShapeShape distance(BRepBuilderAPI_MakeVertex(p).Vertex(),boundary);Near r;
 if(!distance.IsDone()||distance.NbSolution()==0)throw std::runtime_error("distance failed");
 r.d=distance.Value();r.p=distance.PointOnShape2(1);
 std::ostringstream os; os<<std::setprecision(17)<<" support_kind "<<distance.SupportTypeShape2(1);
 TopTools_IndexedMapOfShape em;TopExp::MapShapes(copy,TopAbs_EDGE,em);
 if(distance.SupportTypeShape2(1)==BRepExtrema_IsOnEdge){double t;distance.ParOnEdgeS2(1,t);os<<" edge "<<em.FindIndex(distance.SupportOnShape2(1))<<" t "<<t;}

 for(size_t i=0;i<faces.size();i++){BRepExtrema_DistShapeShape f(BRepBuilderAPI_MakeVertex(r.p).Vertex(),faces[i]);if(f.IsDone()&&f.Value()<1e-6){r.face=i+1;
 BRepAdaptor_Surface surf(faces[i]); GeomAPI_ProjectPointOnSurf proj(r.p,BRep_Tool::Surface(faces[i]));
 os<<" face "<<i+1<<" type "<<surf.GetType()<<" orientation "<<faces[i].Orientation();
 if(proj.NbPoints()){double u,v;proj.LowerDistanceParameters(u,v);os<<" uv "<<u<<' '<<v;}
}}
 r.detail=os.str();return r;
}
std::vector<TopoDS_Face> faces(const TopoDS_Shape&s){std::vector<TopoDS_Face> f;for(TopExp_Explorer it(s,TopAbs_FACE);it.More();it.Next())f.push_back(TopoDS::Face(it.Current()));return f;}
bool bounded(const TopoDS_Face&f,double u,double v){auto s=BRepClass_FaceClassifier(f,gp_Pnt2d(u,v),1e-8).State();return s==TopAbs_IN||s==TopAbs_ON;}
struct Witness{double d=-1,u=0,v=0;gp_Pnt p;Near target;int index=0;std::string kind;};
void diagnose(const std::string&,const Witness&,const TopoDS_Shape&,const TopoDS_Shape&);
void print(const std::string&label,const Witness&w){std::cout<<label<<" distance "<<w.d<<" source_"<<w.kind<<' '<<w.index<<" parameters "<<w.u<<' '<<w.v<<" point ";point(w.p);std::cout<<" target_face "<<w.target.face<<" target_point ";point(w.target.p);std::cout<<w.target.detail;std::cout<<'\n';}
Witness compare(const std::string&label,const TopoDS_Shape&source,const TopoDS_Shape&target,int grid){
 const auto sf=faces(source),tf=faces(target);Witness best;
 auto take=[&](gp_Pnt p,int index,std::string kind,double u,double v){auto n=nearest(p,target,tf);if(n.d>best.d)best={n.d,u,v,p,n,index,kind};};
 for(size_t k=0;k<sf.size();k++){
  double a,b,c,d;BRepTools::UVBounds(sf[k],a,b,c,d);BRepAdaptor_Surface s(sf[k]);
  std::cout<<label<<" source_face "<<k+1<<" type "<<s.GetType()<<" orientation "<<sf[k].Orientation()<<" uv_bounds "<<a<<' '<<b<<' '<<c<<' '<<d<<'\n';
  Witness previous=best;best=Witness{};
  for(int i=0;i<=grid;i++)for(int j=0;j<=grid;j++){double u=a+(b-a)*i/grid,v=c+(d-c)*j/grid;if(bounded(sf[k],u,v))take(s.Value(u,v),k+1,"face",u,v);}
  print(label+" face_coarse",best);
  if(best.d>1e-3) {
   double du=(b-a)/grid,dv=(d-c)/grid;
   for(int round=0;round<5;round++){double u=best.u,v=best.v;for(int i=-4;i<=4;i++)for(int j=-4;j<=4;j++){double x=std::clamp(u+du*i/4,a,b),y=std::clamp(v+dv*j/4,c,d);if(bounded(sf[k],x,y))take(s.Value(x,y),k+1,"face",x,y);}du/=4;dv/=4;}
  }
  print(label+" face_refined",best);if(best.d>1e-4)diagnose(label,best,source,target);
  if(previous.d>best.d)best=previous;
 }
 Witness faceBest=best;print(label+" face_maximum",faceBest);best=Witness{};
 TopTools_IndexedMapOfShape edges;TopExp::MapShapes(source,TopAbs_EDGE,edges);
 for(int k=1;k<=edges.Extent();k++){BRepAdaptor_Curve c(TopoDS::Edge(edges(k)));double a=c.FirstParameter(),b=c.LastParameter();for(int i=0;i<=4*grid;i++){double t=a+(b-a)*i/(4*grid);take(c.Value(t),k,"edge",t,0);}}
 print(label+" coarse",best);
 // Refine around the strongest sampled source location, retaining the source chart.
 if(best.kind=="face"){
  auto f=sf[best.index-1];BRepAdaptor_Surface s(f);double a,b,c,d;BRepTools::UVBounds(f,a,b,c,d);double du=(b-a)/grid,dv=(d-c)/grid;
  for(int round=0;round<5;round++){double u=best.u,v=best.v;for(int i=-4;i<=4;i++)for(int j=-4;j<=4;j++){double x=std::clamp(u+du*i/4,a,b),y=std::clamp(v+dv*j/4,c,d);if(bounded(f,x,y))take(s.Value(x,y),best.index,"face",x,y);}du/=4;dv/=4;}
 }else{BRepAdaptor_Curve c(TopoDS::Edge(edges(best.index)));double a=c.FirstParameter(),b=c.LastParameter(),dt=(b-a)/(4*grid);for(int r=0;r<5;r++){double t=best.u;for(int i=-8;i<=8;i++){double x=std::clamp(t+dt*i/8,a,b);take(c.Value(x),best.index,"edge",x,0);}dt/=4;}}
 print(label+" edge_refined",best);if(faceBest.d>best.d)best=faceBest;print(label+" refined",best);std::cout<<label<<" source_incident"<<nearest(best.p,source,sf).detail<<'\n';return best;
}
const gp_Vec A(0,2,10), Z(0,0,1), M=gp_Vec(0,-10,2).Normalized(), W(0,-1,0);
struct Frame {gp_Pnt c;gp_Vec n,v,cp;double alpha;};
Frame frame(double t){
 gp_Vec d(6+12*t*t,30-84*t+36*t*t,0),dd(24*t,-84+72*t,0);
 auto cross=A.Crossed(d),n=cross.Normalized(),crossp=A.Crossed(dd);
 auto np=(crossp-n*n.Dot(crossp))/cross.Magnitude();
 auto c=gp_Pnt(6*t+4*t*t*t,30*t-42*t*t+12*t*t*t,0).Translated(A*((9.5+.5*n.Z())/10)-n*.5);
 return {gp_Pnt(c.X()*10,c.Y()*10,c.Z()*10),n,gp_Vec(n.X(),n.Y(),0).Normalized(),d*10+A*(.5*np.Z())-np*5,std::acos(n.Z())};
}
double root(std::function<double(double)> f,double lo,double hi){
 if(f(lo)*f(hi)>0)throw std::runtime_error("unbracketed H1 root");
 for(int i=0;i<60;i++){double m=(lo+hi)/2;if(f(m)*f(lo)>0)lo=m;else hi=m;}return (lo+hi)/2;
}
gp_Vec radial(double t,double phi){return Z*std::cos(phi)+frame(t).v*std::sin(phi);}
double margin(gp_Pnt p){
 double x=p.X()/10,t=root([&](double t){return 6*t+4*t*t*t-std::clamp(x,0.,10.);},0,1),y=p.Y()-.2*p.Z();
 return std::min({p.X(),100-p.X(),p.Z(),100-p.Z(),y,10*(30*t-42*t*t+12*t*t*t)-y});
}
void metadata(std::string label,const TopoDS_Shape&s){
 for(auto kind:{TopAbs_FACE,TopAbs_EDGE,TopAbs_VERTEX}){double tol=0;for(TopExp_Explorer it(s,kind);it.More();it.Next()){
 double t=kind==TopAbs_FACE?BRep_Tool::Tolerance(TopoDS::Face(it.Current())):kind==TopAbs_EDGE?BRep_Tool::Tolerance(TopoDS::Edge(it.Current())):BRep_Tool::Tolerance(TopoDS::Vertex(it.Current()));tol=std::max(tol,t);}
 std::cout<<label<<" tolerance_kind "<<kind<<" max "<<tol<<'\n';}
}
void diagnose(const std::string&label,const Witness&w,const TopoDS_Shape&source,const TopoDS_Shape&target){
 const double ts=root([](double t){return gp_Vec(gp_Pnt(0,0,0),frame(t).c).Dot(M)+5;},.8,1);
 auto residual=[&](double t){return gp_Vec(frame(t).c,w.p).Dot(frame(t).cp);};
 for(int i=0;i<100;i++){
 double lo=ts*i/100,hi=ts*(i+1)/100;if(residual(lo)*residual(hi)>0)continue;
 double t=root(residual,lo,hi);auto f=frame(t);auto r=gp_Vec(f.c,w.p);double phi=std::atan2(r.Dot(f.v),r.Dot(Z));
 if(phi<0||phi>f.alpha)continue;
 auto normal=r.Normalized();auto p=f.c.Translated(normal*5);
 std::cout<<label<<" canal_equation source_face "<<w.index<<" t "<<t<<" phi "<<phi<<" envelope_residual "<<residual(t)<<" radius_residual "<<r.Magnitude()-5<<" projection ";point(p);
 std::cout<<" source_margin "<<margin(p)<<" source_distance "<<nearest(p,source,faces(source)).d<<" target_distance "<<nearest(p,target,faces(target)).d;
 for(double off:{-.0005,.0005,-.01,.01}){auto q=p.Translated(normal*off);std::cout<<" offset "<<off<<" source "<<state(source,q)<<" target "<<state(target,q);}std::cout<<'\n';
 }
 auto f=frame(ts);auto dir=gp_Vec(f.c,w.p).Normalized();double det=Z.Dot(M.Crossed(f.n));
 std::cout<<label<<" sphere_equation source_face "<<w.index<<" radius_residual "<<w.p.Distance(f.c)-5<<" cone "<<dir.Dot(M.Crossed(f.n))/det<<' '<<Z.Dot(dir.Crossed(f.n))/det<<' '<<Z.Dot(M.Crossed(dir))/det<<'\n';
}
#ifndef H1_WITNESS_LIBRARY
int main(int argc,char**argv){
 if(argc<3)return 2;
 TopoDS_Shape native;BRep_Builder builder;if(!BRepTools::Read(native,argv[1],builder))return 2;
 STEPControl_Reader reader;if(reader.ReadFile(argv[2])!=IFSelect_RetDone||!reader.TransferRoots())return 2;auto truck=reader.OneShape();
 std::cout<<std::unitbuf<<std::setprecision(17)<<"scale 10 radius 5 classifier_tolerance 1e-7 face_trim_tolerance 1e-8 offset 0.01\n";
 metadata("native",native);metadata("truck",truck);
 const auto nf=faces(native),tf=faces(truck);
 double ts=root([](double t){return gp_Vec(gp_Pnt(0,0,0),frame(t).c).Dot(M)+5;},.8,1);
 auto f=frame(ts);std::cout<<"right_station "<<ts<<" center ";point(f.c);std::cout<<'\n';
 if(argc<5||std::string(argv[4])!="material-only"){
 for(auto w:{compare("native_to_truck",native,truck,argc>3?std::stoi(argv[3]):20),compare("truck_to_native",truck,native,argc>3?std::stoi(argv[3]):20)}){
 std::cout<<"maximum_sphere_equation residual "<<w.p.Distance(f.c)-5<<" point ";point(w.p);std::cout<<'\n';
 auto normal=gp_Vec(f.c,w.p).Normalized();auto p=f.c.Translated(normal*5);
 std::cout<<"maximum_sphere_projection ";point(p);std::cout<<" source_margin "<<margin(p)<<" native_distance "<<nearest(p,native,nf).d<<" truck_distance "<<nearest(p,truck,tf).d;
 for(double off:{-.01,.01}){auto q=p.Translated(normal*off);std::cout<<" offset "<<off<<" native "<<state(native,q)<<" truck "<<state(truck,q);}std::cout<<'\n';
 }
 }
 auto witness=[&](std::string label,double t,double phi,gp_Pnt p,gp_Vec normal){
 auto nn=nearest(p,native,nf),nt=nearest(p,truck,tf);
 std::cout<<"witness "<<label<<" t "<<t<<" phi "<<phi<<" xyz ";point(p);std::cout<<" normal ";point(gp_Pnt(normal.X(),normal.Y(),normal.Z()));
 std::cout<<" source_margin "<<margin(p)<<" native_distance "<<nn.d<<" native_xyz ";point(nn.p);std::cout<<nn.detail<<" truck_distance "<<nt.d<<" truck_xyz ";point(nt.p);std::cout<<nt.detail;
 for(double off:{-.01,.01}){auto q=p.Translated(normal*off);std::cout<<" offset "<<off<<" xyz ";point(q);std::cout<<" native "<<state(native,q)<<" truck "<<state(truck,q);}std::cout<<'\n';
 };
 double beta=std::acos(M.Z()),line=(-5-95*M.Z())/M.Dot(W);
 for(int i=1;i<16;i++){
 double phi=beta*i/16.;double t=root([&](double t){auto f=frame(t);return gp_Vec(gp_Pnt(0,0,0),f.c).Dot(W)-line+5*std::sin(phi)*(f.v.Dot(W)-1);},0,.0625);
 // Interior bisector of the two canal normals at the P-Q intersection.
 auto normal=(radial(t,phi)+Z*std::cos(phi)+W*std::sin(phi)).Normalized();
 witness("P-Q",t,phi,frame(t).c.Translated(radial(t,phi)*5),normal);
 phi=beta+(frame(0).alpha-beta)*i/16.;t=root([&](double t){return gp_Vec(gp_Pnt(0,0,0),frame(t).c.Translated(radial(t,phi)*5)).Dot(M);},0,.0625);
 witness("Q-R",t,phi,frame(t).c.Translated(radial(t,phi)*5),(radial(t,phi)+M).Normalized());
 }
 for(double t:{.03,.1,.3,.5,.7,.85,.94})for(double q:{.2,.5,.8})witness("canal",t,q*frame(t).alpha,frame(t).c.Translated(radial(t,q*frame(t).alpha)*5),radial(t,q*frame(t).alpha));
 for(int i=1;i<7;i++)for(int j=1;j<7;j++){
 auto dir=(Z*i+M*j+f.n*3).Normalized();witness("sphere",ts,double(i)/j,f.c.Translated(dir*5),dir);
 }
 double gamma=std::acos(f.n.Dot(M));auto tangent=(M-f.n*std::cos(gamma))/std::sin(gamma);
 for(double q:{.1,.3,.5,.7,.9}){
 auto dir=f.n*std::cos(q*gamma)+tangent*std::sin(q*gamma);auto top=f.c.Translated(dir*5);
 witness("sphere-ruling-junction",ts,q,top,dir);
 for(double z:{.05,10.,50.,90.}){auto p=top.Translated(A*((z-top.Z())/10));witness("ruling-cylinder",ts,q,p,dir);}
 auto bottom=top.Translated(A*(-top.Z()/10));
 witness("bottom-edge",ts,q,bottom,(dir-Z).Normalized());
 auto interior=bottom.Translated(dir*-.03);interior.SetZ(0);
 witness("bottom-interior",ts,q,interior,-Z);
 }
 for(double z:{1.,30.,60.,90.}){
 auto p=gp_Pnt(0,.2*z,z);witness("original-left-ruling",0,z,p,(frame(0).n+M).Normalized());
 }
}

#endif
