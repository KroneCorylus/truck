#include <BRepClass3d_SolidClassifier.hxx>
#include <STEPControl_Reader.hxx>
#include <cstdlib>
#include <iostream>

// Classify a local material witness in a freshly exported Truck STEP solid.
int main(int argc, char** argv) {
    if (argc != 5) return 2;
    STEPControl_Reader reader;
    if (reader.ReadFile(argv[1]) != IFSelect_RetDone) return 3;
    if (!reader.TransferRoots()) return 4;
    BRepClass3d_SolidClassifier classifier(reader.OneShape(),
        gp_Pnt(std::atof(argv[2]), std::atof(argv[3]), std::atof(argv[4])), 1e-8);
    std::cout << "STATE " << classifier.State() << '\n';
}
