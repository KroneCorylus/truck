// Native BREP inputs, independent deep copies, capture settings and physical selection.
#include <BRepAdaptor_Curve.hxx>
#include <BRepAdaptor_Surface.hxx>
#include <BRepBuilderAPI_Copy.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepFilletAPI_MakeChamfer.hxx>
#include <BRepFilletAPI_MakeFillet.hxx>
#include <BRepTools.hxx>
#include <BRep_Builder.hxx>
#include <GeomAPI_ProjectPointOnCurve.hxx>
#include <BRep_Tool.hxx>
#include <OSD_ThreadPool.hxx>
#include <STEPControl_Writer.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopTools_IndexedMapOfShape.hxx>
#include <TopoDS.hxx>
#include <chrono>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <memory>
#include <sstream>
#include <stdexcept>
#include <vector>
struct Selection { int mode; double a,b; gp_Pnt origin; gp_Vec normal; std::vector<gp_Pnt> points; };
std::string serial(const TopoDS_Shape& s) { std::ostringstream o; BRepTools::Write(s,o); return o.str(); }
void require(bool b,const char* why) { if(!b) throw std::runtime_error(why); }
int main(int argc,char** argv) {
 try {
  require(argc==5,"input.brep config samples|smoke output-prefix");
  bool smoke=std::string(argv[3])=="smoke"; int samples=smoke?2:std::stoi(argv[3]);require(samples>0,"samples");
  OSD_ThreadPool::DefaultPool()->Init(1);
  std::ifstream cfg(argv[2]); int count; cfg>>count; std::vector<Selection> selections;
  for(int j=0;j<count;++j) {
   Selection s;double x,y,z,nx,ny,nz;int n;
   cfg>>s.mode>>s.a>>s.b>>x>>y>>z>>nx>>ny>>nz>>n;s.origin=gp_Pnt(x,y,z);s.normal=gp_Vec(nx,ny,nz);
   for(int k=0;k<n;++k){cfg>>x>>y>>z;s.points.emplace_back(x,y,z);} if(s.points.size()==2){auto a=s.points.front(),b=s.points.back();s.points.insert(s.points.begin()+1,gp_Pnt((a.X()+b.X())/2.,(a.Y()+b.Y())/2.,(a.Z()+b.Z())/2.));} selections.push_back(s);
  }
  require(bool(cfg),"config parse");
  TopoDS_Shape source;BRep_Builder reader;require(BRepTools::Read(source,argv[1],reader),"BREP import");
  const auto original=serial(source);std::string reference,copy_reference;std::cout<<std::setprecision(17);
  for(int i=0;i<=samples;++i) {
   BRepBuilderAPI_Copy copy(source,true,false);const auto input=copy.Shape();
   const auto copy_serial=serial(input);
   if(i==0) {
    copy_reference=copy_serial;
    for(TopExp_Explorer it(source,TopAbs_EDGE);it.More();it.Next()) {
     BRepAdaptor_Curve a(TopoDS::Edge(it.Current())),b(TopoDS::Edge(copy.ModifiedShape(it.Current())));
     require(a.FirstParameter()==b.FirstParameter()&&a.LastParameter()==b.LastParameter(),"copy changed curve domain");
     for(int j=0;j<=16;++j){double t=a.FirstParameter()+(a.LastParameter()-a.FirstParameter())*j/16.;require(a.Value(t).Distance(b.Value(t))<1e-12,"copy changed curve");}
    }
    for(TopExp_Explorer it(source,TopAbs_FACE);it.More();it.Next()) {
     auto f=TopoDS::Face(it.Current());BRepAdaptor_Surface a(f),b(TopoDS::Face(copy.ModifiedShape(f)));
     double u0,u1,v0,v1;BRepTools::UVBounds(f,u0,u1,v0,v1);
     for(int j=0;j<=4;++j)for(int k=0;k<=4;++k){double u=u0+(u1-u0)*j/4.,v=v0+(v1-v0)*k/4.;require(a.Value(u,v).Distance(b.Value(u,v))<1e-12,"copy changed surface");}
    }
   }else require(copy_serial==copy_reference,"nonrepeatable original copy");
   TopTools_IndexedMapOfShape edges;TopExp::MapShapes(input,TopAbs_EDGE,edges);
   std::vector<TopoDS_Edge> chosen;std::vector<TopoDS_Face> faces;
   for(const auto& s:selections) {
    int matches=0;TopoDS_Edge selected;
    for(int k=1;k<=edges.Extent();++k) {
     auto e=TopoDS::Edge(edges(k));BRepAdaptor_Curve c(e);auto p=c.Value(c.FirstParameter()),q=c.Value(c.LastParameter());
     if(!((p.Distance(s.points.front())<1e-6&&q.Distance(s.points.back())<1e-6)||(q.Distance(s.points.front())<1e-6&&p.Distance(s.points.back())<1e-6)))continue;
     double a,b;auto curve=BRep_Tool::Curve(e,a,b);bool fits=true;
     for(auto witness:s.points){GeomAPI_ProjectPointOnCurve project(witness,curve,a,b);if(!project.NbPoints()||project.LowerDistance()>1e-6){fits=false;break;}}
     if(fits){++matches;selected=e;}
    }
    require(matches==1,"selection-unmatched or ambiguous");
    for(auto e:chosen)require(!e.IsSame(selected),"duplicate seed");chosen.push_back(selected);
    TopoDS_Face face;int face_matches=0;
    if(s.mode>=2)for(TopExp_Explorer it(input,TopAbs_FACE);it.More();it.Next()) {
     auto f=TopoDS::Face(it.Current());bool incident=false;for(TopExp_Explorer e(f,TopAbs_EDGE);e.More();e.Next())if(e.Current().IsSame(selected))incident=true;
     if(!incident)continue;BRepAdaptor_Surface surf(f);
     if(surf.GetType()==GeomAbs_Plane){auto p=surf.Plane();if(gp_Vec(p.Axis().Direction()).Crossed(s.normal).Magnitude()<1e-8&&std::abs(gp_Vec(s.origin,p.Location()).Dot(s.normal))<1e-6){face=f;++face_matches;}}
    }
    if(s.mode>=2)require(face_matches==1,"reference-face-unmatched");faces.push_back(face);
   }
   std::chrono::steady_clock::time_point start;if(!smoke)start=std::chrono::steady_clock::now();
   std::unique_ptr<BRepFilletAPI_MakeFillet> fillet;
   std::unique_ptr<BRepFilletAPI_MakeChamfer> chamfer;TopoDS_Shape output;
   if(selections[0].mode==0) {
    fillet=std::make_unique<BRepFilletAPI_MakeFillet>(input,ChFi3d_Rational);
    fillet->SetParams(1e-2,1e-4,1e-5,1e-4,1e-5,1e-3);fillet->SetContinuity(GeomAbs_C1,1e-2);
    for(size_t k=0;k<chosen.size();++k)fillet->Add(selections[k].a,chosen[k]);
    fillet->Build();require(fillet->IsDone(),"operation-failed");output=fillet->Shape();
   }else{
    chamfer=std::make_unique<BRepFilletAPI_MakeChamfer>(input);
    for(size_t k=0;k<chosen.size();++k){auto s=selections[k];if(s.mode==1)chamfer->Add(s.a,chosen[k]);else if(s.mode==2)chamfer->Add(s.a,s.b,chosen[k],faces[k]);else chamfer->AddDA(s.a,s.b,chosen[k],faces[k]);}
    chamfer->Build();require(chamfer->IsDone(),"operation-failed");output=chamfer->Shape();
   }
   double ms=0;if(!smoke)ms=std::chrono::duration<double,std::milli>(std::chrono::steady_clock::now()-start).count();
   auto result=serial(output);
   if(i==0){reference=result;BRepTools::Write(output,(std::string(argv[4])+".brep").c_str());STEPControl_Writer w;require(w.Transfer(output,STEPControl_AsIs)==IFSelect_RetDone,"STEP transfer");require(w.Write((std::string(argv[4])+".step").c_str())==IFSelect_RetDone,"STEP export");}else require(result==reference,"nonrepeatable-output");
   if(!BRepCheck_Analyzer(output,true,false,true).IsValid()) {
    BRepTools::Write(output,(std::string(argv[4])+".invalid.brep").c_str());
    std::cerr<<"invalid-output: exact BRepCheck failed; ordinary BRepCheck="<<BRepCheck_Analyzer(output).IsValid()<<"\n";
    throw std::runtime_error("native QA blocker; no timing result");
   }
   require(serial(source)==original,"mutated-source");
   std::cout<<"{\"engine\":\"occt\",\"sample\":"<<i<<",\"warmup\":"<<(i==0?"true":"false")<<",\"smoke\":"<<(smoke?"true":"false");if(!smoke)std::cout<<",\"ms\":"<<ms;std::cout<<",\"repeatable\":true,\"valid\":true}"<<std::endl;
  }
 }catch(const std::exception& e){std::cerr<<e.what()<<std::endl;return 1;}
}
