// Sampled G6 localization and independent polynomial rolling-ball witnesses.
// Capture coordinates are ten times the source script. No fitted center is used.
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
struct Frame {gp_Pnt c; gp_Vec n,v,cp; double alpha;};
Frame frame(double t) {
 const double x=80*t-120*t*t+100*t*t*t-50*std::pow(t,4), y=30*t*t-20*t*t*t+10*std::pow(t,4);
 const gp_Vec a(0,2,10),d(80-240*t+300*t*t-200*t*t*t,60*t-60*t*t+40*t*t*t,0);
 const gp_Vec dd(-240+600*t-600*t*t,60-120*t+120*t*t,0);
 const auto cross=d.Crossed(a),crossp=dd.Crossed(a),n=cross.Normalized();
 const auto np=(crossp-n*n.Dot(crossp))/cross.Magnitude();
 const double h=(9+n.Z())/10;
 const auto c=gp_Pnt(x,y,0).Translated(a*h-n);
 return {gp_Pnt(c.X()*10,c.Y()*10,c.Z()*10),n,gp_Vec(n.X(),n.Y(),0).Normalized(),d*10+a*np.Z()-np*10,std::acos(n.Z())};
}
gp_Vec wall(){return gp_Vec(-10,5,-1).Normalized();}
double station(){double lo=0,hi=.5;for(int i=0;i<60;i++){double t=(lo+hi)/2;if(gp_Vec(gp_Pnt(0,0,0),frame(t).c).Dot(wall())+10>0)lo=t;else hi=t;}return (lo+hi)/2;}
void point(const gp_Pnt&p){std::cout<<'['<<p.X()<<','<<p.Y()<<','<<p.Z()<<']';}
const char* state(const TopoDS_Shape&s,const gp_Pnt&p){auto t=BRepClass3d_SolidClassifier(s,p,1e-7).State();return t==TopAbs_IN?"IN":t==TopAbs_OUT?"OUT":t==TopAbs_ON?"ON":"UNKNOWN";}
double source_violation(const gp_Pnt& p) {
 const double y=p.Y()-.2*p.Z(); double lo=0,hi=1;
 for(int i=0;i<60;i++){double t=(lo+hi)/2;double yt=10*(30*t*t-20*t*t*t+10*std::pow(t,4));if(yt<y)lo=t;else hi=t;}
 const double t=(lo+hi)/2,x=10*(80*t-120*t*t+100*t*t*t-50*std::pow(t,4));
 return std::max({-p.Z(),p.Z()-100,-y,y-200,.5*y-p.X(),p.X()-x});
}
struct Near{double d=1e100;gp_Pnt p;int face=0;};
Near nearest(const gp_Pnt&p,const TopoDS_Shape& target,const std::vector<TopoDS_Face>&faces){
 TopoDS_Compound boundary; BRep_Builder builder; builder.MakeCompound(boundary);
 const auto copy=BRepBuilderAPI_Copy(target,false,false).Shape();
 for(TopExp_Explorer it(copy,TopAbs_FACE);it.More();it.Next())builder.Add(boundary,it.Current());
 // Query boundary faces, never the solid interior (which would report zero).
 BRepExtrema_DistShapeShape distance(BRepBuilderAPI_MakeVertex(p).Vertex(),boundary);Near r;
 if(!distance.IsDone()||distance.NbSolution()==0)throw std::runtime_error("distance failed");
 r.d=distance.Value();r.p=distance.PointOnShape2(1);
 for(size_t i=0;i<faces.size();i++){BRepExtrema_DistShapeShape f(BRepBuilderAPI_MakeVertex(r.p).Vertex(),faces[i]);if(f.IsDone()&&f.Value()<1e-6){r.face=i+1;break;}}
 return r;
}
std::vector<TopoDS_Face> faces(const TopoDS_Shape&s){std::vector<TopoDS_Face> f;for(TopExp_Explorer it(s,TopAbs_FACE);it.More();it.Next())f.push_back(TopoDS::Face(it.Current()));return f;}
bool bounded(const TopoDS_Face&f,double u,double v){auto s=BRepClass_FaceClassifier(f,gp_Pnt2d(u,v),1e-8).State();return s==TopAbs_IN||s==TopAbs_ON;}
struct Witness{double d=-1,u=0,v=0;gp_Pnt p;Near target;int index=0;std::string kind;};
void print(const std::string&label,const Witness&w){std::cout<<label<<" distance "<<w.d<<" source_"<<w.kind<<' '<<w.index<<" parameters "<<w.u<<' '<<w.v<<" point ";point(w.p);std::cout<<" target_face "<<w.target.face<<" target_point ";point(w.target.p);std::cout<<'\n';}
Witness compare(const std::string&label,const TopoDS_Shape&source,const TopoDS_Shape&target,int grid){
 const auto sf=faces(source),tf=faces(target);Witness best;
 auto take=[&](gp_Pnt p,int index,std::string kind,double u,double v){auto n=nearest(p,target,tf);if(n.d>best.d)best={n.d,u,v,p,n,index,kind};};
 for(size_t k=0;k<sf.size();k++){
  double a,b,c,d;BRepTools::UVBounds(sf[k],a,b,c,d);BRepAdaptor_Surface s(sf[k]);
  Witness previous=best;best=Witness{};
  for(int i=0;i<=grid;i++)for(int j=0;j<=grid;j++){double u=a+(b-a)*i/grid,v=c+(d-c)*j/grid;if(bounded(sf[k],u,v))take(s.Value(u,v),k+1,"face",u,v);}
  print(label+" face_coarse",best);
  if(best.d>1e-3) {
   double du=(b-a)/grid,dv=(d-c)/grid;
   for(int round=0;round<5;round++){double u=best.u,v=best.v;for(int i=-4;i<=4;i++)for(int j=-4;j<=4;j++){double x=std::clamp(u+du*i/4,a,b),y=std::clamp(v+dv*j/4,c,d);if(bounded(sf[k],x,y))take(s.Value(x,y),k+1,"face",x,y);}du/=4;dv/=4;}
  }
  print(label+" face_refined",best);
  if(best.d>1e-3 && best.p.Y()>50 && best.p.Z()>80) {
   auto residual=[&](double t){auto f=frame(t);return gp_Vec(f.c,best.p).Dot(f.cp);};
   double lo=.05,hi=.99;const double flo=residual(lo);
   if(flo*residual(hi)>=0)throw std::runtime_error("canal station not bracketed");
   for(int i=0;i<60;i++){double t=(lo+hi)/2;if(residual(t)*flo>0)lo=t;else hi=t;}
   double t=(lo+hi)/2;auto f=frame(t);auto radial=gp_Vec(f.c,best.p);auto normal=radial.Normalized();
   auto midpoint=f.c.Translated(normal*((radial.Magnitude()+10)/2));
   std::cout<<label<<" localized_canal station "<<t<<" radius_residual "<<radial.Magnitude()-10<<" source_violation "<<source_violation(midpoint)<<" midpoint ";point(midpoint);
   std::cout<<" source_state "<<state(source,midpoint)<<" target_state "<<state(target,midpoint)<<'\n';
  }
  if(previous.d>best.d)best=previous;
 }
 TopTools_IndexedMapOfShape edges;TopExp::MapShapes(source,TopAbs_EDGE,edges);
 for(int k=1;k<=edges.Extent();k++){BRepAdaptor_Curve c(TopoDS::Edge(edges(k)));double a=c.FirstParameter(),b=c.LastParameter();for(int i=0;i<=4*grid;i++){double t=a+(b-a)*i/(4*grid);take(c.Value(t),k,"edge",t,0);}}
 print(label+" coarse",best);
 // Refine around the strongest sampled source location, retaining the source chart.
 if(best.kind=="face"){
  auto f=sf[best.index-1];BRepAdaptor_Surface s(f);double a,b,c,d;BRepTools::UVBounds(f,a,b,c,d);double du=(b-a)/grid,dv=(d-c)/grid;
  for(int round=0;round<5;round++){double u=best.u,v=best.v;for(int i=-4;i<=4;i++)for(int j=-4;j<=4;j++){double x=std::clamp(u+du*i/4,a,b),y=std::clamp(v+dv*j/4,c,d);if(bounded(f,x,y))take(s.Value(x,y),best.index,"face",x,y);}du/=4;dv/=4;}
 }else{BRepAdaptor_Curve c(TopoDS::Edge(edges(best.index)));double a=c.FirstParameter(),b=c.LastParameter(),dt=(b-a)/(4*grid);for(int r=0;r<5;r++){double t=best.u;for(int i=-8;i<=8;i++){double x=std::clamp(t+dt*i/8,a,b);take(c.Value(x),best.index,"edge",x,0);}dt/=4;}}
 print(label+" refined",best);return best;
}
int main(int argc,char**argv){
 if(argc>=4 && std::string(argv[1])=="--construct") {
  STEPControl_Reader reader;if(reader.ReadFile(argv[2])!=IFSelect_RetDone||!reader.TransferRoots())return 2;
  const auto source=reader.OneShape();BRepFilletAPI_MakeFillet fillet(source,ChFi3d_Rational);
  const double factor=argc>4?std::stod(argv[4]):1.;
  fillet.SetParams(1e-2,1e-4,1e-5,1e-4/factor,1e-5,1e-3);fillet.SetContinuity(GeomAbs_C1,1e-2);
  TopTools_IndexedMapOfShape edges;TopExp::MapShapes(source,TopAbs_EDGE,edges);
  for(const auto& p:{gp_Pnt(193.75,76.25,100),gp_Pnt(0,10,50),gp_Pnt(50,120,100)}) {
   int found=0;for(int i=1;i<=edges.Extent();i++){auto e=TopoDS::Edge(edges(i));BRepAdaptor_Curve c(e);if(c.Value((c.FirstParameter()+c.LastParameter())/2).Distance(p)<1e-6){fillet.Add(10.,e);found++;}}
   if(found!=1)throw std::runtime_error("ambiguous G6 selection");
  }
  fillet.Build();if(!fillet.IsDone())throw std::runtime_error("native G6 failed");
  BRepTools::Write(fillet.Shape(),argv[3]);std::cout<<"native_construct_valid "<<BRepCheck_Analyzer(fillet.Shape()).IsValid()<<'\n';return 0;
 }
 if(argc<3){std::cerr<<"usage: g6_corner_witness native.brep truck.step [grid=24]\n";return 2;}
 TopoDS_Shape native;BRep_Builder builder;if(!BRepTools::Read(native,argv[1],builder))return 2;
 STEPControl_Reader reader;if(reader.ReadFile(argv[2])!=IFSelect_RetDone||!reader.TransferRoots())return 2;auto truck=reader.OneShape();
 const int grid=argc>3?std::stoi(argv[3]):24;std::cout<<std::setprecision(17);
 const double t=station();const auto f=frame(t);std::cout<<"quartic_station "<<t<<" center ";point(f.c);std::cout<<'\n';
 const auto nw=compare("native_to_truck",native,truck,grid);compare("truck_to_native",truck,native,grid);
 const gp_Vec n(0,0,1),m=wall();const double det=n.Dot(m.Crossed(f.n));
 for(auto entry:{std::make_pair("native",native),std::make_pair("truck",truck)}){
  int index=0;for(auto face:faces(entry.second)){index++;BRepAdaptor_Surface s(face);double a,b,c,d;BRepTools::UVBounds(face,a,b,c,d);int count=0;double worst=0,mincone=1e100,maxsource=-1e100;gp_Pnt wp;
   for(int i=0;i<=80;i++)for(int j=0;j<=80;j++){double u=a+(b-a)*i/80,v=c+(d-c)*j/80;if(!bounded(face,u,v))continue;auto p=s.Value(u,v);auto vcp=gp_Vec(f.c,p);if(vcp.Magnitude()<1e-8)continue;auto dir=vcp.Normalized();
    const double c0=dir.Dot(m.Crossed(f.n))/det,c1=n.Dot(dir.Crossed(f.n))/det,c2=n.Dot(m.Crossed(dir))/det;
    if(std::min({c0,c1,c2}) < -1e-7 || p.Distance(f.c)>11)continue;
    count++;mincone=std::min(mincone,std::min({c0,c1,c2}));double err=std::abs(p.Distance(f.c)-10);if(count==1||err>worst){worst=err;wp=p;}
    maxsource=std::max(maxsource,source_violation(p));
   }
   if(entry.first==std::string("truck") && count>100 && (worst>1e-6 || maxsource>1e-7))throw std::runtime_error("Truck corner violates polynomial material witness");
   if(count){std::cout<<entry.first<<" cone_face "<<index<<" type "<<s.GetType()<<" samples "<<count<<" sphere_residual "<<worst<<" min_cone "<<mincone<<" max_source_violation "<<maxsource<<" worst ";point(wp);std::cout<<'\n';}
  }
 }
 auto dir=gp_Vec(f.c,nw.p).Normalized();
 const auto midpoint=f.c.Translated(dir*((nw.p.Distance(f.c)+10)/2));
 const double y=midpoint.Y()-.2*midpoint.Z();double lo=0,hi=1;
 for(int i=0;i<60;i++){double t=(lo+hi)/2,yt=10*(30*t*t-20*t*t*t+10*std::pow(t,4));if(yt<y)lo=t;else hi=t;}
 const double st=(lo+hi)/2, profile_x=10*(80*st-120*st*st+100*st*st*st-50*std::pow(st,4));
 std::cout<<"localized_positive_cone "<<dir.Dot(m.Crossed(f.n))/det<<' '<<n.Dot(dir.Crossed(f.n))/det<<' '<<n.Dot(m.Crossed(dir))/det<<'\n';
 std::cout<<"localized_source_margins wall_x "<<midpoint.X()-.5*y<<" profile_x "<<profile_x-midpoint.X()<<" y "<<y<<" ymax "<<200-y<<" z "<<midpoint.Z()<<" zmax "<<100-midpoint.Z()<<'\n';
std::cout<<"localized_native_radius "<<nw.p.Distance(f.c)<<'\n';
 for(double radius:{9.99,(nw.p.Distance(f.c)+10)/2,10.01}){auto p=f.c.Translated(dir*radius);std::cout<<"radial_section radius "<<radius<<" point ";point(p);std::cout<<" native "<<state(native,p)<<" truck "<<state(truck,p)<<'\n';}
 if(source_violation(midpoint)>1e-7 || dir.Dot(m.Crossed(f.n))/det<=0 || n.Dot(dir.Crossed(f.n))/det<=0 || n.Dot(m.Crossed(dir))/det<=0)throw std::runtime_error("localized corner outside source/cone");
 if(std::string(state(truck,f.c.Translated(dir*9.99)))!="IN" || std::string(state(truck,f.c.Translated(dir*10.01)))!="OUT")throw std::runtime_error("Truck corner section failed");
 // Independent canal interior points (station, angle fraction), and two sides.
 const auto nf=faces(native),tf=faces(truck);
 for(double station:{.1,.25,.5,.6,.61,.617,.625,.75,.95})for(double fraction:{.2,.3,.5,.8}){auto q=frame(station);auto normal=n*std::cos(q.alpha*fraction)+q.v*std::sin(q.alpha*fraction);auto p=q.c.Translated(normal*10);auto nn=nearest(p,native,nf),nt=nearest(p,truck,tf);
 if(nt.d>1e-4 || std::string(state(truck,p.Translated(normal*-.01)))!="IN" || std::string(state(truck,p.Translated(normal*.01)))!="OUT")throw std::runtime_error("Truck canal material witness failed");
 std::cout<<"canal "<<station<<' '<<fraction<<" point ";point(p);std::cout<<" native_distance "<<nn.d<<" face "<<nn.face<<" truck_distance "<<nt.d<<" face "<<nt.face<<" inward "<<state(native,p.Translated(normal*-.01))<<'/'<<state(truck,p.Translated(normal*-.01))<<" outward "<<state(native,p.Translated(normal*.01))<<'/'<<state(truck,p.Translated(normal*.01))<<'\n';}
}
