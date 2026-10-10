#include <BRepAlgoAPI_Common.hxx>
#include <BRepAlgoAPI_Cut.hxx>
#include <BRepBuilderAPI_Copy.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepFilletAPI_MakeFillet.hxx>
#include <BRepGProp.hxx>
#include <BRepMesh_IncrementalMesh.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRepPrimAPI_MakePrism.hxx>
#include <BRepAdaptor_Curve.hxx>
#include <BRep_Tool.hxx>
#include <BRep_Builder.hxx>
#include <GProp_GProps.hxx>
#include <OSD_Parallel.hxx>
#include <OSD_ThreadPool.hxx>
#include <Poly_Triangulation.hxx>
#include <STEPControl_Reader.hxx>
#include <Standard_Version.hxx>
#include <TopExp_Explorer.hxx>
#include <TopTools_ListOfShape.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Compound.hxx>
#include <gp_Circ.hxx>
#include <algorithm>
#include <array>
#include <chrono>
#include <cmath>
#include <cstdlib>
#include <fstream>
#include <functional>
#include <iomanip>
#include <iostream>
#include <map>
#include <sstream>
#include <stdexcept>
#include <string>
#include <vector>

constexpr double pi = 3.14159265358979323846;
double fuzzy = 0.0;
bool parallel = false;
using Triangle = std::array<gp_Pnt, 3>;
struct Output {
  TopoDS_Shape shape;
  std::vector<Triangle> triangles;
  bool isMesh = false;
};
struct Case {
  std::function<Output()> run;
  double expected;
  double qaTolerance;
};

void require(bool condition, const std::string& message) {
  if (!condition) throw std::runtime_error(message);
}

std::string quote(const std::string& s) {
  std::ostringstream out;
  out << '"';
  for (unsigned char c : s) {
    if (c == '"' || c == '\\') out << '\\' << c;
    else if (c < 32) out << "\\u" << std::hex << std::setw(4) << std::setfill('0') << int(c);
    else out << c;
  }
  out << '"';
  return out.str();
}

TopoDS_Shape cylinder(double r, double height, double x = 0, double y = 0, double z = 0) {
  return BRepPrimAPI_MakeCylinder(gp_Ax2(gp_Pnt(x,y,z),gp_Dir(0,0,1)), r, height).Shape();
}

TopoDS_Shape compound(const std::vector<TopoDS_Shape>& shapes) {
  BRep_Builder builder;
  TopoDS_Compound result;
  builder.MakeCompound(result);
  for (const auto& shape : shapes) builder.Add(result, shape);
  return result;
}

template<class Algorithm>
TopoDS_Shape boolean(const TopoDS_Shape& a, const TopoDS_Shape& b) {
  Algorithm operation;
  TopTools_ListOfShape arguments, tools;
  arguments.Append(a);
  tools.Append(b);
  operation.SetArguments(arguments);
  operation.SetTools(tools);
  operation.SetNonDestructive(true);
  operation.SetRunParallel(parallel);
  operation.SetFuzzyValue(fuzzy);
  operation.Build();
  if (operation.HasErrors() || !operation.IsDone()) {
    std::ostringstream errors;
    operation.DumpErrors(errors);
    throw std::runtime_error("boolean: " + errors.str());
  }
  return operation.Shape();
}

std::vector<Triangle> mesh(const TopoDS_Shape& body, double tol) {
  BRepMesh_IncrementalMesh mesher(body, tol, false, pi, parallel);
  require(mesher.IsDone(), "tessellation failed");
  std::vector<Triangle> result;
  for (TopExp_Explorer it(body, TopAbs_FACE); it.More(); it.Next()) {
    const auto face = TopoDS::Face(it.Current());
    TopLoc_Location location;
    const auto triangulation = BRep_Tool::Triangulation(face, location);
    require(!triangulation.IsNull() && triangulation->NbTriangles()>0, "missing mesh face");
    for (int i=1; i<=triangulation->NbTriangles(); ++i) {
      int a,b,c;
      triangulation->Triangle(i).Get(a,b,c);
      if (face.Orientation() == TopAbs_REVERSED) std::swap(b,c);
      result.push_back({triangulation->Node(a).Transformed(location.Transformation()),
                        triangulation->Node(b).Transformed(location.Transformation()),
                        triangulation->Node(c).Transformed(location.Transformation())});
    }
  }
  return result;
}

TopoDS_Wire circle(double x, double y, double r) {
  return BRepBuilderAPI_MakeWire(BRepBuilderAPI_MakeEdge(
    gp_Circ(gp_Ax2(gp_Pnt(x,y,0),gp_Dir(0,0,1)),r)).Edge()).Wire();
}

Case fixture(const std::string& name, double tol, double qaTol, const std::string& directory) {
  const auto slash = name.find('/');
  require(slash != std::string::npos, "case/parameter required");
  const auto kind = name.substr(0,slash);
  const auto parameter = name.substr(slash+1);
  const double value = std::stod(parameter);
  const int n = static_cast<int>(value);
  if (kind=="holes_batch" || kind=="holes_sequential" || kind=="mesh_plate") {
    const int cols=static_cast<int>(std::ceil(std::sqrt(n))), rows=(n+cols-1)/cols;
    const double expected=cols*rows*32.0-n*pi*2.0;
    auto plate=BRepPrimAPI_MakeBox(cols*4.0,rows*4.0,2.0).Shape();
    std::vector<TopoDS_Shape> cutters;
    for(int i=0;i<n;++i) cutters.push_back(cylinder(1,4,(i%cols)*4+2,(i/cols)*4+2,-1));
    if(kind=="mesh_plate") {
      BRepBuilderAPI_MakePolygon rectangle;
      for (const auto& p : std::vector<gp_Pnt>{{0,0,0},{cols*4.0,0,0},{cols*4.0,rows*4.0,0},{0,rows*4.0,0}}) rectangle.Add(p);
      rectangle.Close();
      BRepBuilderAPI_MakeFace face(rectangle.Wire());
      for(int i=0;i<n;++i) { auto hole=circle((i%cols)*4+2,(i/cols)*4+2,1); hole.Reverse(); face.Add(hole); }
      plate=BRepPrimAPI_MakePrism(face.Face(),gp_Vec(0,0,2)).Shape();
      return {[plate,tol] {
        const auto copy=BRepBuilderAPI_Copy(plate,false,false).Shape();
        return Output{copy,mesh(copy,tol),true};
      },expected,tol};
    }
    const auto batch=compound(cutters);
    return {[plate,cutters,batch,kind] {
      auto result=plate;
      if(kind=="holes_sequential") for(const auto& cutter:cutters) result=boolean<BRepAlgoAPI_Cut>(result,cutter);
      else result=boolean<BRepAlgoAPI_Cut>(plate,batch);
      return Output{result,{},false};
    },expected,qaTol};
  }
  if(kind=="knurl") {
    const double r=12, depth=.35, half=pi/(2*n), height=8;
    const double outerX=r-depth+1.25*(r*std::cos(half)-(r-depth)), outerY=1.25*r*std::sin(half);
    require(outerX>r && std::atan2(outerY,outerX)<pi/n,"knurl tools must clear the stock and remain disjoint");
    std::vector<TopoDS_Shape> cutters;
    for(int i=0;i<n;++i) {
      const double angle=2*pi*i/n;
      BRepBuilderAPI_MakePolygon polygon;
      for(const auto& xy:std::vector<std::array<double,2>>{{r-depth,0},
          {outerX,-outerY}, {outerX,outerY}}) {
        polygon.Add(gp_Pnt(xy[0]*std::cos(angle)-xy[1]*std::sin(angle),xy[0]*std::sin(angle)+xy[1]*std::cos(angle),-1));
      }
      polygon.Close();
      cutters.push_back(BRepPrimAPI_MakePrism(BRepBuilderAPI_MakeFace(polygon.Wire()).Face(),gp_Vec(0,0,10)).Shape());
    }
    const auto body=cylinder(r,height), tool=compound(cutters);
    return {[body,tool] {return Output{boolean<BRepAlgoAPI_Cut>(body,tool),{},false};},
      height*(pi*r*r-n*(r*r*half-(r-depth)*r*std::sin(half))),qaTol};
  }
  if(kind=="thread") {
    STEPControl_Reader reader;
    require(reader.ReadFile((directory+"/thread-"+std::to_string(n)+".step").c_str())==IFSelect_RetDone,"read thread STEP");
    require(reader.TransferRoots()>0,"transfer thread STEP");
    const auto tool=reader.OneShape();
    require(BRepCheck_Analyzer(tool,true,false,true).IsValid(),"invalid imported thread operand");
    const auto body=cylinder(4,n*1.25);
    const double r=4, root=3.6, pitch=1.25;
    const double radialArea=(r*r-root*root)/2;
    const double radialMoment=(r*r*r-root*root*root)/3-root*radialArea;
    return {[body,tool] {return Output{boolean<BRepAlgoAPI_Cut>(body,tool),{},false};},
      n*(pi*r*r*pitch-4*pi*(pitch/16*radialArea+radialMoment/std::sqrt(3.0))),qaTol};
  }
  if(kind=="near_tangent") {
    const double distance=2-value;
    const auto a=cylinder(1,2), b=cylinder(1,2,distance);
    return {[a,b] {return Output{boolean<BRepAlgoAPI_Common>(a,b),{},false};},
      2*(2*std::acos(distance/2)-distance/2*std::sqrt(4-distance*distance)),std::max(1e-6,std::min(qaTol,value/10000))};
  }
  if(kind=="thin_wall") {
    const auto body=cylinder(4,8), tool=cylinder(4-value,10,0,0,-1);
    return {[body,tool] {return Output{boolean<BRepAlgoAPI_Cut>(body,tool),{},false};},
      pi*(16-(4-value)*(4-value))*8,std::min(qaTol,value/1000)};
  }
  if(kind=="fillet_bore") {
    const auto body=boolean<BRepAlgoAPI_Cut>(BRepPrimAPI_MakeBox(gp_Pnt(-10,-10,0),20,20,10).Shape(),cylinder(3,12,0,0,-1));
    TopoDS_Edge edge;
    for(TopExp_Explorer it(body,TopAbs_EDGE);it.More();it.Next()) {
      BRepAdaptor_Curve curve(TopoDS::Edge(it.Current()));
      if(curve.GetType()==GeomAbs_Circle && std::abs(curve.Circle().Location().Z()-10)<1e-6) {edge=TopoDS::Edge(it.Current());break;}
    }
    require(!edge.IsNull(),"missing bore rim");
    const double removed=2*pi*(3*value*value*(1-pi/4)+value*value*value*(5.0/6-pi/4));
    return {[body,edge,value] {
      BRepFilletAPI_MakeFillet fillet(body);
      fillet.Add(value,edge);
      fillet.Build();
      require(fillet.IsDone(),"bore fillet failed");
      return Output{fillet.Shape(),{},false};
    },4000-90*pi-removed,qaTol};
  }
  throw std::runtime_error("unknown case " + name);
}

void check(Output output, double expected, double tol) {
  int solids=0, faces=0;
  for(TopExp_Explorer it(output.shape,TopAbs_SOLID);it.More();it.Next()) ++solids;
  for(TopExp_Explorer it(output.shape,TopAbs_FACE);it.More();it.Next()) ++faces;
  const bool geometric=BRepCheck_Analyzer(output.shape,true,false,true).IsValid();
  GProp_GProps properties;
  BRepGProp::VolumeProperties(output.shape,properties,1e-9,true,false);
  const double exactVolume=properties.Mass();
  if(!output.isMesh) output.triangles=mesh(output.shape,tol);
  using Vertex=std::array<long long,3>;
  using Edge=std::pair<Vertex,Vertex>;
  std::map<Edge,std::pair<int,int>> edges;
  double volume=0;
  for(const auto& triangle:output.triangles) {
    std::array<Vertex,3> vertices;
    for(int i=0;i<3;++i) vertices[i]={std::llround(triangle[i].X()/1e-8),std::llround(triangle[i].Y()/1e-8),std::llround(triangle[i].Z()/1e-8)};
    if(vertices[0]==vertices[1] || vertices[1]==vertices[2] || vertices[2]==vertices[0]) continue;
    volume+=triangle[0].XYZ().Dot(triangle[1].XYZ().Crossed(triangle[2].XYZ()))/6;
    for(int i=0;i<3;++i) {
      auto a=vertices[i], b=vertices[(i+1)%3];
      const int sign=a<b?1:-1;
      if(b<a) std::swap(a,b);
      auto& edge=edges[{a,b}]; ++edge.first; edge.second+=sign;
    }
  }
  const bool closed=!edges.empty() && std::all_of(edges.begin(),edges.end(),[](const auto& e){return e.second.first==2 && e.second.second==0;});
  const double error=std::abs(volume-expected)/expected;
  const double exactError=std::abs(exactVolume-expected)/expected;
  const bool pass=solids==1 && geometric && closed && std::isfinite(volume) && volume>0 && error<=.002 && exactError<=.002;
  std::cout << "RESULT {\"phase\":\"qa\",\"qa\":{\"pass\":" << (pass?"true":"false")
    << ",\"geometric\":" << (geometric?"true":"false") << ",\"mesh_closed\":" << (closed?"true":"false")
    << ",\"solids\":" << solids << ",\"faces\":" << faces << ",\"triangles\":" << output.triangles.size()
    << ",\"volume\":" << volume << ",\"exact_volume\":" << exactVolume << ",\"expected_volume\":" << expected
    << ",\"relative_volume_error\":" << error << ",\"exact_relative_volume_error\":" << exactError << ",\"qa_tolerance\":" << tol << "}}" << std::endl;
}

long peakRss() {
  std::ifstream status("/proc/self/status");
  std::string line;
  while(std::getline(status,line)) if(line.rfind("VmHWM:",0)==0) return std::stol(line.substr(6));
  return 0;
}

int main(int argc,char** argv) {
  std::cout << std::setprecision(17);
  if(argc==2 && std::string(argv[1])=="--version") {std::cout << OCC_VERSION_COMPLETE << std::endl;return 0;}
  try {
    require(argc==6,"CASE SAMPLES TOL QA_TOL FIXTURE_DIR");
    const int samples=std::stoi(argv[2]);
    const double tol=std::stod(argv[3]), qaTol=std::stod(argv[4]);
    require(samples>=0 && std::isfinite(tol) && tol>0 && std::isfinite(qaTol) && qaTol>0,"invalid settings");
    const int workers=std::getenv("RAYON_NUM_THREADS")?std::stoi(std::getenv("RAYON_NUM_THREADS")):1;
    parallel=workers>1;
    fuzzy=std::getenv("OCCT_FUZZY")?std::stod(std::getenv("OCCT_FUZZY")):0;
    require(workers>0 && std::isfinite(fuzzy) && fuzzy>=0,"invalid threads/fuzzy tolerance");
    OSD_Parallel::SetUseOcctThreads(true);
    OSD_ThreadPool::DefaultPool(workers)->Init(workers);
    const auto test=fixture(argv[1],tol,qaTol,argv[5]);
    auto output=test.run();
    std::vector<double> times;
    for(int i=0;i<samples;++i) {
      const auto start=std::chrono::steady_clock::now();
      auto next=test.run();
      const auto end=std::chrono::steady_clock::now();
      times.push_back(std::chrono::duration<double,std::milli>(end-start).count());
      output=std::move(next);
    }
    std::cout << "RESULT {\"phase\":\"timing\",\"peak_rss_kib\":" << peakRss() << ",\"samples_ms\":[";
    for(size_t i=0;i<times.size();++i) {if(i) std::cout << ',';std::cout << times[i];}
    std::cout << "]}" << std::endl;
    check(std::move(output),test.expected,test.qaTolerance);
  } catch(const Standard_Failure& e) {
    std::cout << "RESULT {\"phase\":\"failure\",\"error\":" << quote(e.GetMessageString()?e.GetMessageString():"OCCT failure") << "}" << std::endl;
    return 1;
  } catch(const std::exception& e) {
    std::cout << "RESULT {\"phase\":\"failure\",\"error\":" << quote(e.what()) << "}" << std::endl;
    return 1;
  }
}
