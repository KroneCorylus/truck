// Captured DRAW configuration; all import, copy, selection and QA are untimed.
// G6 matched construction; independent witnesses are run separately. Input coordinates are capture scale10.
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
 int faces=0;for(TopExp_Explorer it(s,TopAbs_FACE);it.More();it.Next())++faces;
 std::cout<<"QA valid 1 faces "<<faces<<" volume "<<props.Mass()<<'\n';
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
    for(const auto& p:{gp_Pnt(193.75,76.25,100),gp_Pnt(0,10,50),gp_Pnt(50,120,100)}) {
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
