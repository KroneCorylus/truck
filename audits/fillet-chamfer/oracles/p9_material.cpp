#include <STEPControl_Reader.hxx>
#include <BRepClass3d_SolidClassifier.hxx>
#include <iostream>

// Classify independent material witnesses against the exported Truck artifact.
int main(int argc, char** argv) {
    if (argc != 2) return 2;
    STEPControl_Reader reader;
    if (reader.ReadFile(argv[1]) != IFSelect_RetDone) return 3;
    reader.TransferRoots();
    const auto shape = reader.OneShape();
    double x, y, z;
    while (std::cin >> x >> y >> z) {
        BRepClass3d_SolidClassifier classifier(shape, gp_Pnt(x, y, z), 1e-8);
        std::cout << "STATE " << int(classifier.State()) << '\n';
    }
}
