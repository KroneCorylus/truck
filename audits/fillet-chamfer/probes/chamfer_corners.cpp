#include <Geom_BSplineSurface.hxx>
#include <BRepAdaptor_Curve.hxx>
#include <BRepAdaptor_Surface.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepFilletAPI_MakeChamfer.hxx>
#include <BRepGProp.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRepTools.hxx>
#include <BRepTools_WireExplorer.hxx>
#include <BRep_Tool.hxx>
#include <GProp_GProps.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopTools_IndexedMapOfShape.hxx>
#include <TopTools_IndexedDataMapOfShapeListOfShape.hxx>
#include <TopoDS.hxx>
#include <cstdlib>
#include <iomanip>
#include <iostream>
#include <sstream>
#include <string>
#include <vector>

// usage: probe box|qcyl  then triples "edgeIndex d1 d2 faceIndex" (d2<0 => equal distance)
static const char* stype(GeomAbs_SurfaceType t) {
  switch (t) { case GeomAbs_Plane: return "plane"; case GeomAbs_Cylinder: return "cyl";
    case GeomAbs_Cone: return "cone"; case GeomAbs_Sphere: return "sphere"; case GeomAbs_Torus: return "torus";
    case GeomAbs_BSplineSurface: return "bspline"; default: return "other"; }
}
static const char* ctype(GeomAbs_CurveType t) {
  switch (t) { case GeomAbs_Line: return "line"; case GeomAbs_Circle: return "circle"; case GeomAbs_Ellipse: return "ellipse";
    case GeomAbs_BSplineCurve: return "bspline"; default: return "other"; }
}
int main(int argc, char** argv) {
  std::string kind = argv[1];
  TopoDS_Shape input = kind == "box" ? BRepPrimAPI_MakeBox(10, 10, 10).Shape()
                                     : BRepPrimAPI_MakeCylinder(gp_Ax2(gp_Pnt(0,0,0), gp_Dir(1,0,0)), 1, 1, M_PI/2).Shape();
  TopTools_IndexedMapOfShape edges, faces;
  TopExp::MapShapes(input, TopAbs_EDGE, edges);
  TopExp::MapShapes(input, TopAbs_FACE, faces);
  std::cout << std::setprecision(12);
  if (argc == 2) {
    for (int i = 1; i <= edges.Extent(); ++i) {
      TopoDS_Vertex a, b; TopExp::Vertices(TopoDS::Edge(edges(i)), a, b);
      auto p = BRep_Tool::Pnt(a), q = BRep_Tool::Pnt(b);
      std::cout << "edge " << i << " " << ctype(BRepAdaptor_Curve(TopoDS::Edge(edges(i))).GetType()) << " (" << p.X() << "," << p.Y() << "," << p.Z() << ")->(" << q.X() << "," << q.Y() << "," << q.Z() << ")\n";
    }
    for (int i = 1; i <= faces.Extent(); ++i) {
      std::cout << "face " << i << " " << stype(BRepAdaptor_Surface(TopoDS::Face(faces(i))).GetType());
      for (TopExp_Explorer v(faces(i), TopAbs_VERTEX); v.More(); v.Next()) { auto p = BRep_Tool::Pnt(TopoDS::Vertex(v.Current())); std::cout << " (" << p.X() << "," << p.Y() << "," << p.Z() << ")"; }
      std::cout << "\n";
    }
    return 0;
  }
  BRepFilletAPI_MakeChamfer chamfer(input);
  for (int k = 2; k + 3 < argc + 1; k += 4) {
    int e = std::atoi(argv[k]); double d1 = std::atof(argv[k+1]), d2 = std::atof(argv[k+2]); int f = std::atoi(argv[k+3]);
    if (d2 < 0) chamfer.Add(d1, TopoDS::Edge(edges(e)));
    else chamfer.Add(d1, d2, TopoDS::Edge(edges(e)), TopoDS::Face(faces(f)));
  }
  chamfer.Build();
  if (!chamfer.IsDone()) { std::cout << "failed\n"; return 1; }
  auto result = chamfer.Shape();
  GProp_GProps props; BRepGProp::VolumeProperties(result, props, 1e-10, true, false);
  int nf = 0; for (TopExp_Explorer it(result, TopAbs_FACE); it.More(); it.Next()) ++nf;
  std::cout << "valid=" << BRepCheck_Analyzer(result).IsValid() << " faces=" << nf << " volume=" << props.Mass() << "\n";
  for (TopExp_Explorer it(result, TopAbs_FACE); it.More(); it.Next()) {
    auto face = TopoDS::Face(it.Current());
    bool original = false;
    for (int i = 1; i <= faces.Extent(); ++i) if (chamfer.Modified(faces(i)).Contains(face) || faces(i).IsSame(face)) original = true;
    std::cout << (original ? "ORIG " : "NEW  ") << stype(BRepAdaptor_Surface(face).GetType()) << ":";
    for (BRepTools_WireExplorer w(BRepTools::OuterWire(face), face); w.More(); w.Next()) {
      auto p = BRep_Tool::Pnt(w.CurrentVertex());
      std::cout << " [" << ctype(BRepAdaptor_Curve(w.Current()).GetType()) << "](" << p.X() << "," << p.Y() << "," << p.Z() << ")";
    }
    std::cout << "\n";
    if (getenv("SAMPLE") && BRepAdaptor_Surface(face).GetType() == GeomAbs_BSplineSurface) {
      double a, b, c, d; BRepTools::UVBounds(face, a, b, c, d);
      BRepAdaptor_Surface S(face);
      for (double u : {0., .25, .5, .75, 1.}) { for (double v : {0., .25, .5, .75, 1.}) { auto p = S.Value(a + (b - a) * u, c + (d - c) * v); std::cout << "  S(" << u << "," << v << ")=(" << p.X() << "," << p.Y() << "," << p.Z() << ")"; } std::cout << "\n"; }
      auto bs = S.BSpline(); std::cout << "  degree " << bs->UDegree() << "x" << bs->VDegree() << " poles " << bs->NbUPoles() << "x" << bs->NbVPoles() << "\n";
    }
  }
}
