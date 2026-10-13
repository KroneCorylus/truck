#include <BRepTools.hxx>
#include <BRep_Tool.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <BRep_Builder.hxx>
#include <BRepGProp.hxx>
#include <BRepClass3d_SolidClassifier.hxx>
#include <GProp_GProps.hxx>
#include <iomanip>
#include <iostream>

int main(int argc, char** argv) {
  if (argc != 2) return 2;
  TopoDS_Shape shape;
  BRep_Builder builder;
  if (!BRepTools::Read(shape, argv[1], builder)) return 3;
  std::cout << std::setprecision(17);
  GProp_GProps regular, kronrod;
  BRepGProp::VolumeProperties(shape, regular, 1e-9, true, false);
  double error = BRepGProp::VolumePropertiesGK(shape, kronrod, 1e-9, true, true);
  std::cout << "volume " << regular.Mass() << " GK " << kronrod.Mass() << " error " << error << '\n';
  for (auto type : {TopAbs_VERTEX, TopAbs_EDGE, TopAbs_FACE}) {
    int i=0;
    for (TopExp_Explorer ex(shape,type);ex.More();ex.Next()) {
      double t = type==TopAbs_VERTEX ? BRep_Tool::Tolerance(TopoDS::Vertex(ex.Current())) : type==TopAbs_EDGE ? BRep_Tool::Tolerance(TopoDS::Edge(ex.Current())) : BRep_Tool::Tolerance(TopoDS::Face(ex.Current()));
      if(t>1e-5) std::cout << "tolerance " << type << " " << i << " " << t << "\n";
      ++i;
    }
  }
  BRepClass3d_SolidClassifier classifier(shape);
  for (double z : {-90., 0., 99.8}) {
    for (double x : {-5., 1., 5., 20.}) {
      for (double y : {-5., -1., 5.}) {
        classifier.Perform(gp_Pnt(x,y,z), 1e-7);
        std::cout << x << ' ' << y << ' ' << z << ' ' << classifier.State() << '\n';
      }
    }
  }
}
