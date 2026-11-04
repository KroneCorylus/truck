#include <BRepClass3d_SolidClassifier.hxx>
#include <BRepTools.hxx>
#include <BRep_Builder.hxx>
#include <STEPControl_Reader.hxx>
#include <iostream>
#include <string>

// Read retained BREP or generated STEP and classify stdin triples without meshing.
int main(int argc, char** argv) {
    if (argc != 3) return 2;
    TopoDS_Shape shape;
    if (std::string(argv[1]).ends_with(".step")) {
        STEPControl_Reader reader;
        if (reader.ReadFile(argv[1]) != IFSelect_RetDone || !reader.TransferRoots()) return 3;
        shape = reader.OneShape();
    } else {
        BRep_Builder builder;
        if (!BRepTools::Read(shape, argv[1], builder)) return 4;
    }
    double x, y, z;
    while (std::cin >> x >> y >> z) {
        BRepClass3d_SolidClassifier classifier(shape, gp_Pnt(x, y, z), 1e-8);
        std::cout << classifier.State() << '\n';
    }
}
