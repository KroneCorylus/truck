#include <STEPControl_Reader.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepGProp.hxx>
#include <BRep_Tool.hxx>
#include <GProp_GProps.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <algorithm>
#include <cmath>
#include <cstdlib>
#include <iostream>

int main(int argc, char** argv) {
    if (argc != 3) return 2;
    STEPControl_Reader reader;
    if (reader.ReadFile(argv[1]) != IFSelect_RetDone || !reader.TransferRoots()) return 3;
    const auto shape = reader.OneShape();
    const bool valid = BRepCheck_Analyzer(shape).IsValid();
    GProp_GProps props;
    BRepGProp::VolumePropertiesGK(shape, props, 1e-9, true, true);
    double max_tolerance = 0.;
    int solids = 0;
    for (TopExp_Explorer ex(shape, TopAbs_SOLID); ex.More(); ex.Next()) ++solids;
    for (TopExp_Explorer ex(shape, TopAbs_EDGE); ex.More(); ex.Next())
        max_tolerance = std::max(max_tolerance, BRep_Tool::Tolerance(TopoDS::Edge(ex.Current())));
    for (TopExp_Explorer ex(shape, TopAbs_VERTEX); ex.More(); ex.Next())
        max_tolerance = std::max(max_tolerance, BRep_Tool::Tolerance(TopoDS::Vertex(ex.Current())));
    const double expected = std::strtod(argv[2], nullptr);
    std::cout.precision(17);
    std::cout << "valid " << valid << "\nsolids " << solids
              << "\nvolume " << props.Mass() << "\nexpected " << expected
              << "\nabsolute_error " << std::abs(props.Mass() - expected)
              << "\nmax_edge_vertex_tolerance " << max_tolerance << '\n';
    return valid && solids == 1 && std::abs(props.Mass() - expected) < 0.01
        && max_tolerance < 0.001 ? 0 : 4;
}
