#include <BRepClass3d_SolidClassifier.hxx>
#include <STEPControl_Reader.hxx>
#include <TopAbs_State.hxx>
#include <TopoDS_Shape.hxx>

#include <algorithm>
#include <array>
#include <cmath>
#include <iomanip>
#include <iostream>
#include <stdexcept>

static TopoDS_Shape read(const char* path) {
    STEPControl_Reader reader;
    if (reader.ReadFile(path) != IFSelect_RetDone || reader.TransferRoots() != 1)
        throw std::runtime_error("STEP fixture did not transfer exactly one root");
    auto shape = reader.OneShape();
    if (shape.IsNull()) throw std::runtime_error("empty STEP fixture");
    return shape;
}

int main(int argc, char** argv) {
    try {
        if (argc != 4) throw std::runtime_error("usage: f9_classifier original.step chart10.step source.step");
        const std::array<TopoDS_Shape, 3> shapes{read(argv[1]), read(argv[2]), read(argv[3])};
        const std::array<const char*, 3> names{"original", "chart10", "source"};
        const double k = std::sqrt(26.0), D = 50 / k, low = 250 / k, high = 770 / k;
        const double U = high - 5 - k, dc = 1 + 5 / k;
        double left = -100, right = 0;
        for (int i = 0; i < 64; ++i) {
            const double t = (left + right) / 2, p = -t / (100 * k);
            if ((50 - t * t / 200) / k - 1 / std::hypot(1.0, p) - 1 > 0)
                right = t;
            else
                left = t;
        }
        const double t = (left + right) / 2, p = -t / (100 * k);
        const double Y = t + p / std::hypot(1.0, p), dt = (50 - t * t / 200) / k;
        std::array<int, 3> failures{};
        std::cout << std::setprecision(17);
        for (int j = 1; j < 100; ++j) {
            const double depth = dc * j / 100, q = std::sqrt(2 * depth - depth * depth);
            const double u = U + q, y = std::sqrt(10000 - 200 * k * depth);
            const double slope = y / (100 * k), norm = std::hypot(slope, 1.0);
            const double nu = q, ny = slope / norm, nd = depth - 1 + 1 / norm;
            const double length = std::sqrt(nu * nu + ny * ny + nd * nd);
            for (int sign : {-1, 1}) {
                // Original units are 1/10 capture units; the capture offset is 0.0001.
                const double uu = u + sign * 0.00001 * nu / length;
                const double yy = y + sign * 0.00001 * ny / length;
                const double dd = depth + sign * 0.00001 * nd / length;
                if (!(dd > 0 && dd < D && dd < 2))
                    throw std::runtime_error("probe outside the independent equation domain");
                const double extent = std::sqrt(10000 - 200 * k * dd);
                const double chord = std::sqrt(2 * dd - dd * dd);
                const double lower_y = dd <= dt ? Y - chord : -extent;
                const double upper_u = dd <= dc ? U + chord : high - 5 * dd;
                const std::array<double, 6> source_slack{
                    dd, D - dd, yy + extent, extent - yy, uu - (low - 5 * dd), high - 5 * dd - uu};
                const std::array<double, 6> retained_slack{
                    dd, D - dd, yy - lower_y, extent - yy, uu - (low - 5 * dd), upper_u - uu};
                const bool expected_inside = sign < 0;
                const double source_margin = *std::min_element(source_slack.begin(), source_slack.end());
                const double retained_margin = *std::min_element(retained_slack.begin(), retained_slack.end());
                if ((source_margin > 0) != expected_inside || (retained_margin > 0) != expected_inside)
                    throw std::runtime_error("independent material inequalities disagree with expected side");
                const gp_Pnt point(10 * ((uu + 5 * dd) / k - 250.0 / 26),
                                   10 * yy, 10 * ((5 * uu - dd) / k + 50.0 / 26));
                const auto expected = expected_inside ? TopAbs_IN : TopAbs_OUT;
                for (std::size_t i = 0; i < shapes.size(); ++i) {
                    BRepClass3d_SolidClassifier classifier(shapes[i], point, 1e-7);
                    const auto state = classifier.State();
                    failures[i] += state != expected;
                    std::cout << "{\"station\":" << j << ",\"sign\":" << sign
                              << ",\"representation\":\"" << names[i] << "\",\"expected\":" << expected
                              << ",\"actual\":" << state << ",\"xyz\":[" << point.X() << ','
                              << point.Y() << ',' << point.Z() << "],\"source_margin\":" << source_margin
                              << ",\"retained_margin\":" << retained_margin << "}\n";
                }
            }
        }
        std::cout << "{\"checks_per_representation\":198,\"failures\":["
                  << failures[0] << ',' << failures[1] << ',' << failures[2] << "]}\n";
        return std::any_of(failures.begin(), failures.end(), [](int n) { return n != 0; }) ? 1 : 0;
    } catch (const std::exception& error) {
        std::cerr << error.what() << '\n';
        return 2;
    }
}
