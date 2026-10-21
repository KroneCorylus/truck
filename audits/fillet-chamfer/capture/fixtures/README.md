# Native volume controls

The curved-cap input and chamfer are byte-identical BREP exports of
chamfer/equal_dist_sequence/C1 capture 001 in the clean
2026-10-20-adaptive-chamfer-contacts archive. They retain their native tolerances.
The exact spline prism in test_native_volume.cpp has x(u)=u. Its volume is the
integral of its cubic B-spline height, computed from knot spans and control values.
Native input and result meshes are separate diagnostics, not exact volume oracles;
the native result has a retained mesh/integral discrepancy in the proof archive.

`test_native_distance.cpp` also uses the curved-cap chamfer. A tiny rectangular
patch cut from face 3's original B-spline support lies strictly inside its trim,
so its directed distance to the original shape is exactly zero. The original
gradient query reports about 0.009279; rotated and scaled copies exercise the
same invariant. Analytic controls independently check boundary distances and
trim classification, including a planar Bezier face whose topology flag was
previously changed while assembling the comparison boundary.
