#include <BRepTools.hxx>
#include <BRep_Builder.hxx>
#include <BRepAdaptor_Curve.hxx>
#include <BRepAdaptor_Surface.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <BRepExtrema_DistShapeShape.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepGProp.hxx>
#include <GProp_GProps.hxx>
#include <STEPControl_Reader.hxx>
#include <TopExp.hxx>
#include <TopTools_IndexedDataMapOfShapeListOfShape.hxx>
#include <TopTools_IndexedMapOfShape.hxx>
#include <TopoDS.hxx>
#include <gp_Cone.hxx>
#include <gp_Pln.hxx>
#include <algorithm>
#include <cmath>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <stdexcept>
#include <string>
#include <vector>
TopoDS_Shape read(const std::string& path){
 TopoDS_Shape shape;
 if(path.size()>5&&path.substr(path.size()-5)==".step"){
  STEPControl_Reader r;if(r.ReadFile(path.c_str())!=IFSelect_RetDone)throw std::runtime_error("STEP read failed");r.TransferRoots();shape=r.OneShape();
 }else {BRep_Builder b;if(!BRepTools::Read(shape,path.c_str(),b))throw std::runtime_error("BREP read failed");}
 if(shape.IsNull())throw std::runtime_error("null shape");
 return shape;
}
void values(std::ostream& out,const std::vector<double>& a){out<<'[';for(std::size_t i=0;i<a.size();++i){if(i)out<<',';out<<a[i];}out<<']';}
void unique(std::vector<double>& a){std::sort(a.begin(),a.end());a.erase(std::unique(a.begin(),a.end(),[](double x,double y){return std::abs(x-y)<1e-6;}),a.end());}
int main(int argc,char**argv){
 if(argc!=3)return 1;
 std::ifstream tasks(argv[1]);std::ofstream out(argv[2]);out<<std::setprecision(17)<<"[\n";
 std::string input,result;int row;double ax,ay,az,mx,my,mz,bx,by,bz,limit;bool first=true;
 while(tasks>>row>>input>>result>>ax>>ay>>az>>mx>>my>>mz>>bx>>by>>bz>>limit){
  auto shape=read(input);TopTools_IndexedDataMapOfShapeListOfShape adjacency;TopExp::MapShapesAndAncestors(shape,TopAbs_EDGE,TopAbs_FACE,adjacency);
  gp_Pnt a(ax,ay,az),b(bx,by,bz),middle(mx,my,mz);int matches=0;gp_Cone cone;gp_Pln plane;bool gotCone=false,gotPlane=false;
  for(int i=1;i<=adjacency.Extent();++i){
   auto edge=TopoDS::Edge(adjacency.FindKey(i));BRepAdaptor_Curve c(edge);auto x=c.Value(c.FirstParameter()),y=c.Value(c.LastParameter());
   if(std::min(std::max(a.Distance(x),b.Distance(y)),std::max(a.Distance(y),b.Distance(x)))>1e-5)continue;
   BRepExtrema_DistShapeShape distance(BRepBuilderAPI_MakeVertex(middle).Shape(),edge);
   if(!distance.IsDone()||distance.Value()>1e-5)continue;
   ++matches;
   for(const auto& f:adjacency.FindFromIndex(i)){
    BRepAdaptor_Surface s(TopoDS::Face(f));
    if(s.GetType()==GeomAbs_Cone){cone=s.Cone();gotCone=true;}
    if(s.GetType()==GeomAbs_Plane){plane=s.Plane();gotPlane=true;}
   }
  }
  if(matches!=1||!gotCone||!gotPlane)throw std::runtime_error("nonunique plane/cone seed");
  const double theta=std::abs(cone.SemiAngle());gp_Vec axis(cone.Axis().Direction());if(cone.SemiAngle()<0)axis.Reverse();
  const auto apex=cone.Apex();const double h0=gp_Vec(apex,a).Dot(axis);const auto center=apex.Translated(axis*h0);
  const double radius=gp_Vec(center,a).Magnitude();
  if(std::abs(radius-h0*std::tan(theta))>1e-6||gp_Vec(plane.Axis().Direction()).Crossed(axis).Magnitude()>1e-8)throw std::runtime_error("not a perpendicular circular rim");
  auto output=read(result);TopTools_IndexedMapOfShape edges;TopExp::MapShapes(output,TopAbs_EDGE,edges);std::vector<double> planar,conical;double planarSpread=0,coneSpread=0;int planarCircles=0,coneCircles=0;
  for(int i=1;i<=edges.Extent();++i){
   BRepAdaptor_Curve c(TopoDS::Edge(edges(i)));double minR=1e100,maxR=-1e100,minH=1e100,maxH=-1e100,planeResidual=0,coneResidual=0;
   for(int j=0;j<=16;++j){auto p=c.Value(c.FirstParameter()+(c.LastParameter()-c.FirstParameter())*j/16.);gp_Vec v(apex,p);double h=v.Dot(axis),r=(v-axis*h).Magnitude();minR=std::min(minR,r);maxR=std::max(maxR,r);minH=std::min(minH,h);maxH=std::max(maxH,h);planeResidual=std::max(planeResidual,std::abs(h-h0));coneResidual=std::max(coneResidual,std::abs(r-h*std::tan(theta)));}
   double inset=std::abs(radius-(minR+maxR)/2),setback=std::abs((minH+maxH)/2-h0)/std::cos(theta);
   if(planeResidual<1e-6&&maxR-minR<1e-6&&inset>1e-6&&inset<limit){planar.push_back(inset);planarSpread=std::max(planarSpread,maxR-minR);planarCircles+=c.GetType()==GeomAbs_Circle;}
   if(coneResidual<1e-6&&maxH-minH<1e-6&&setback>1e-6&&setback<limit){conical.push_back(setback);coneSpread=std::max(coneSpread,(maxH-minH)/std::cos(theta));coneCircles+=c.GetType()==GeomAbs_Circle;}
  }
  unique(planar);unique(conical);GProp_GProps props;BRepGProp::VolumeProperties(output,props,1e-10,true,false);
  if(!first)out<<",\n";
  first=false;
  out<<"{\"row\":"<<row<<",\"valid\":"<<(BRepCheck_Analyzer(output).IsValid()?"true":"false")<<",\"radius\":"<<radius<<",\"half_angle\":"<<theta<<",\"volume\":"<<props.Mass()<<",\"planar_setbacks\":";values(out,planar);out<<",\"cone_setbacks\":";values(out,conical);out<<",\"planar_spread\":"<<planarSpread<<",\"cone_spread\":"<<coneSpread<<",\"planar_circles\":"<<planarCircles<<",\"cone_circles\":"<<coneCircles<<'}';
 }
 out<<"\n]\n";
}
