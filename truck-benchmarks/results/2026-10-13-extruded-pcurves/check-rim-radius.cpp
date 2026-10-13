#include <STEPControl_Reader.hxx>
#include <BRepAdaptor_Surface.hxx>
#include <BRepClass_FaceClassifier.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <algorithm>
#include <cmath>
#include <cstdlib>
#include <iostream>

int main(int argc, char** argv) {
    if (argc != 3) return 2;
    const bool j8 = std::atoi(argv[2]) == 8;
    const double a = j8 ? 50. : 100., shift = j8 ? 100. : 20.;
    STEPControl_Reader reader;
    if (reader.ReadFile(argv[1]) != IFSelect_RetDone || !reader.TransferRoots()) return 3;
    int faces = 0, samples = 0;
    double radius_error = 0., normal_error = 0.;
    for (TopExp_Explorer ex(reader.OneShape(), TopAbs_FACE); ex.More(); ex.Next()) {
        const auto face = TopoDS::Face(ex.Current());
        BRepAdaptor_Surface surface(face);
        if (surface.GetType() != GeomAbs_BSplineSurface) continue;
        ++faces;
        for (int i = 1; i < 64; ++i) for (int j = 1; j < 64; ++j) {
            const double u = surface.FirstUParameter()
                + (surface.LastUParameter() - surface.FirstUParameter()) * i / 64.;
            const double v = surface.FirstVParameter()
                + (surface.LastVParameter() - surface.FirstVParameter()) * j / 64.;
            BRepClass_FaceClassifier classifier(face, gp_Pnt2d(u, v), 1e-7);
            if (classifier.State() != TopAbs_IN && classifier.State() != TopAbs_ON) continue;
            gp_Pnt p;
            gp_Vec du, dv;
            surface.D1(u, v, p, du, dv);
            gp_Vec n = du.Crossed(dv);
            if (n.Magnitude() < 1e-12) continue;
            n.Normalize();
            if (face.Orientation() == TopAbs_REVERSED) n.Reverse();
            if (j8) {
                p = gp_Pnt(100. - p.Y(), p.Z(), 100. - p.X());
                n = gp_Vec(-n.Y(), n.Z(), -n.X());
            }
            if (std::hypot(n.X(), n.Y()) < 1e-6) continue;
            const double phi = std::atan2(50. * n.Y(), a * n.X());
            gp_Vec lateral(std::cos(phi) / a, std::sin(phi) / 50.,
                           -shift / 100. * std::cos(phi) / a);
            lateral.Normalize();
            const double t = (90. + 10. * lateral.Z()) / 100.;
            const gp_Pnt e(a * std::cos(phi), 50. * std::sin(phi), 0.);
            const gp_Pnt center = e.Translated(t * gp_Vec(shift, 0., 100.) - 10. * lateral);
            gp_Vec radial(center, p);
            radius_error = std::max(radius_error, std::abs(radial.Magnitude() - 10.));
            radial.Normalize();
            normal_error = std::max(normal_error, (radial - n).Magnitude());
            ++samples;
        }
    }
    std::cout.precision(17);
    std::cout << "bspline_faces " << faces << "\nsamples " << samples
              << "\nmax_radius_error " << radius_error
              << "\nmax_normal_vector_error " << normal_error << '\n';
    return samples ? 0 : 4;
}
