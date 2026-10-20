#include <BRepAdaptor_Curve.hxx>
#include <BRepAdaptor_Surface.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeHalfSpace.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepAlgoAPI_Cut.hxx>
#include <BRepFilletAPI_MakeFillet.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepTools.hxx>
#include <BRep_Builder.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Compound.hxx>
#include <gp_Pln.hxx>
#include <cmath>
#include <stdexcept>
int main(int argc,char**argv){
 if(argc!=3)return 1;double angle=std::stod(argv[2])*std::acos(-1.)/180.,h=.782539615254126,k=std::tan(angle);
 auto box=BRepPrimAPI_MakeBox(1.,1.,1.).Shape();TopoDS_Edge vertical;
 for(TopExp_Explorer it(box,TopAbs_EDGE);it.More();it.Next()){auto e=TopoDS::Edge(it.Current());BRepAdaptor_Curve c(e);auto a=c.Value(c.FirstParameter()),b=c.Value(c.LastParameter());if(std::abs(a.X()-1.)<1e-8 && std::abs(b.X()-1.)<1e-8 && std::abs(a.Y())<1e-8 && std::abs(b.Y())<1e-8)vertical=e;}
 if(vertical.IsNull())throw std::runtime_error("missing vertical");BRepFilletAPI_MakeFillet round(box);round.Add(.4,vertical);round.Build();if(!round.IsDone())throw std::runtime_error("first round failed");
 auto plane=BRepBuilderAPI_MakeFace(gp_Pln(gp_Pnt(0,0,h),gp_Dir(k,0,1)),-10.,10.,-10.,10.).Face();auto half=BRepPrimAPI_MakeHalfSpace(plane,gp_Pnt(0,0,2)).Solid();BRepAlgoAPI_Cut cut(round.Shape(),half);cut.Build();if(!cut.IsDone())throw std::runtime_error("cap cut failed");auto solid=cut.Shape();if(!BRepCheck_Analyzer(solid).IsValid())throw std::runtime_error("invalid input");
 TopoDS_Edge seed;TopoDS_Face cap;
 for(TopExp_Explorer it(solid,TopAbs_FACE);it.More();it.Next()){auto f=TopoDS::Face(it.Current());BRepAdaptor_Surface s(f);if(s.GetType()==GeomAbs_Plane && s.Plane().Axis().Direction().Z()>.9 && std::abs(s.Plane().Location().Z())>.5)cap=f;}
 if(cap.IsNull())throw std::runtime_error("missing cap");
 for(TopExp_Explorer it(cap,TopAbs_EDGE);it.More();it.Next()){auto e=TopoDS::Edge(it.Current());BRepAdaptor_Curve c(e);if(c.GetType()==GeomAbs_Circle || c.GetType()==GeomAbs_Ellipse)seed=e;}
 if(seed.IsNull())throw std::runtime_error("missing seed");TopoDS_Compound bundle;BRep_Builder b;b.MakeCompound(bundle);b.Add(bundle,solid);b.Add(bundle,seed);b.Add(bundle,cap);return BRepTools::Write(bundle,argv[1])?0:1;
}
