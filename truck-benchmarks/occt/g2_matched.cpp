// Captured DRAW configuration; all import, copy, selection and QA are untimed.
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

#include <BRepFilletAPI_MakeFillet.hxx>
#include <BRepBuilderAPI_Copy.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepGProp.hxx>
#include <GProp_GProps.hxx>
#include <TopTools_IndexedMapOfShape.hxx>
#include <TopExp.hxx>
#include <OSD_ThreadPool.hxx>
#include <chrono>
#include <sstream>
#include <memory>
#include <vector>

std::string serialized(const TopoDS_Shape& s) { std::ostringstream o; BRepTools::Write(s,o); return o.str(); }
void qa(const TopoDS_Shape& s) {
  if(!BRepCheck_Analyzer(s,true,false,true).IsValid()) throw std::runtime_error("invalid shape");
  GProp_GProps props; BRepGProp::VolumeProperties(s,props,1e-9);
  double worst=0; int count=0, faces=0;
  for(TopExp_Explorer it(s,TopAbs_FACE);it.More();it.Next()) {
    ++faces; if(faces!=8) continue;
    const auto face=TopoDS::Face(it.Current()); BRepAdaptor_Surface surf(face);
    double a,b,c,d; BRepTools::UVBounds(face,a,b,c,d);
    for(int i=0;i<=100;++i)for(int j=0;j<=100;++j) {
      double u=a+(b-a)*i/100.,v=c+(d-c)*j/100.;
      auto st=BRepClass_FaceClassifier(face,gp_Pnt2d(u,v),1e-8).State();
      if(st!=TopAbs_IN&&st!=TopAbs_ON)continue;
      ++count;worst=std::max(worst,std::abs(surf.Value(u,v).Distance(center())-10.));
    }
    std::cout<<"QA corner_type "<<surf.GetType()<<" ";
  }
  std::cout<<"valid 1 faces "<<faces<<" volume "<<props.Mass()<<" radius_residual "<<worst<<" samples "<<count<<" section "<<state(s,gp_Pnt(191.30544979784545,898.00291109619172,1405.5421336081076))<<'\n';
}
int main(int argc,char**argv) {
  if(argc!=5) return 2;
  const int samples=std::stoi(argv[2]);const double factor=std::stod(argv[3]);
  OSD_ThreadPool::DefaultPool()->Init(1);
  STEPControl_Reader reader;if(reader.ReadFile(argv[1])!=IFSelect_RetDone||!reader.TransferRoots())return 2;
  const auto source=reader.OneShape();const auto original=serialized(source);
  std::string reference;std::cout<<std::setprecision(17);
  for(int i=0;i<=samples;++i) {
    BRepBuilderAPI_Copy copy(source,true,false);const auto input=copy.Shape();
    TopTools_IndexedMapOfShape edges;TopExp::MapShapes(input,TopAbs_EDGE,edges);
    std::vector<TopoDS_Edge> selected;
    for(const auto& p:{gp_Pnt(200,0,1000),gp_Pnt(200,0,1500),gp_Pnt(100,1000,1000)}) {
      int matches=0;
      for(int k=1;k<=edges.Extent();++k) {
        auto e=TopoDS::Edge(edges(k)); BRepAdaptor_Curve c(e);
        if(c.Value((c.FirstParameter()+c.LastParameter())/2).Distance(p)<1e-6){selected.push_back(e);++matches;}
      }
      if(matches!=1)throw std::runtime_error("ambiguous selection");
    }
    auto start=std::chrono::steady_clock::now();
    auto builder=std::make_unique<BRepFilletAPI_MakeFillet>(input,ChFi3d_Rational);
    builder->SetParams(1e-2,1e-4,1e-5,1e-4/factor,1e-5,1e-3);
    builder->SetContinuity(GeomAbs_C1,1e-2);
    for(const auto&e:selected)builder->Add(10.,e);
    builder->Build();if(!builder->IsDone())throw std::runtime_error("fillet failed");
    const auto output=builder->Shape();
    const double ms=std::chrono::duration<double,std::milli>(std::chrono::steady_clock::now()-start).count();
    const auto serial=serialized(output);
    if(i==0){reference=serial;qa(output);BRepTools::Write(output,argv[4]);}
    else if(serial!=reference)throw std::runtime_error("nonrepeatable output");
    if(original!=serialized(source))throw std::runtime_error("mutated input");
    std::cout<<"{\"engine\":\"occt\",\"factor\":"<<factor<<",\"sample\":"<<i<<",\"warmup\":"<<(i==0?"true":"false")<<",\"ms\":"<<ms<<"}"<<std::endl;
    if(i==samples)qa(output);
  }
}
