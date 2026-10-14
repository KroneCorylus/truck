# Cusp fillet contacts and rolling-surface inverse searches

This group recovers the radius-0.5 fillet along the vertical cusp of the closed cubic Bézier prism in native `bug30940_3`. Its two adjacent faces share one closed support profile, but use opposite endpoint branches. An unseeded inverse chose the same tangent for both supports and failed to initialize the rolling circle.

The wire fillet now obtains initial parameters from each oriented face's own boundary, only when the guide point is ambiguous at a nonperiodic support seam. `RbfSurface::try_with_support_hints` carries those two finite parameter guesses through contact construction, contact-curve inverses, surface inverses and adjacent-edge crossings. Cutting, boxing, reversing and serializing the contact curves preserve the respective support hints; reversing swaps them with the supports. Default construction and unseeded JSON retain their prior representation. No consumer migration or added dependency is required.

The inverse-search regressions also expose two existing problems on unambiguous supports. An unnormalized scalar triple product made the coplanarity convergence criterion scale-dependent, and the final angular parameter used an approximate iteration circle rather than the evaluated rolling section. The inverse now checks distance to the contact plane and computes the angle from the evaluated section. Singular matrix inversions return failure instead of panicking.

## Independent geometry and regressions

The original prism's section is `x=30t(1-t)^2`, `y=30t^2(1-t)`, with area 15 and height 10. A radius-0.5 ball tangent to the two cusp branches has center `(c,c,z)`. Its contact parameter solves `x-y-r(x'+y')/hypot(x',y')=0`, giving `t=0.01828340462417396` and `c=0.5094782647758623`. Green's theorem replaces the two cubic legs by their tangent circular arc, giving retained volume **149.39834519269158**. The adjacent standalone `independent-volume.py` derives this value without either kernel's output.

The native-input regression checks input immutability, geometric consistency, generated radius and normals, independent material, closed tessellation and STEP round trips. It includes rigid motion, scale and reversed selection. Geometry regressions cover scales 0.01/1/100, both contact inverses, surface inverses, derivatives, both adjacent-edge searches, cutting, in-place and copied reversal, boxing, serialization and rejection of nonfinite hints. Cutting a line guide renormalizes each segment's parameter domain; the test checks the corresponding original physical station. Existing unsupported nontangent-rim regressions remain unchanged.

The recorded parent is `b75dbdba`. The new native fillet regression fails at `find_contact` on that parent. Both standalone inverse regressions also fail against parent production sources: the small-scale inverse misses its evaluated point, and the singular support inverse panics. No existing assertion, tolerance or ignore setting was weakened.

## Validation and scope

Exact commands, final source and executable hashes, release outcomes and targeted native checks are recorded in the adjacent logs and metadata. Raw native inputs and references in the targeted archive are preserved byte for byte from the previous clean capture. Full native capture follows the implementation commit and is archived separately.

This change does not implement nontangent closed-rim fillets, nonplanar multi-edge corners, or collapsing rolling supports. The other four Bézier cases retain their unsupported-operation diagnostics and unchanged inputs. Distance comparisons are sampled evidence, not certified global bounds. Benchmark smoke establishes that the actions run, not a performance improvement.

Final five-crate release validation: **957 passed, one unchanged PCurve precision failure, five existing ignored tests, 152 targets**. The failure remains `truck-geometry/tests/pcurve.rs::pcurve_derivation`, with the previously recorded -2829663063.761872 / -2829663063.7618704 values. All-target release Clippy (including benchmarks), single-threaded action smoke and the final replay build passed.

The final targeted native replay produces one valid solid for `bug30940_3`. Independent Gauss–Kronrod integration gives **149.39834440191035**, an absolute error of **7.908e-7** from the dimension-derived volume. Its maximum edge/vertex tolerance is **2.536e-6**. Native-to-Truck sampled distance is **2.3441845121833323e-6** (572 samples); Truck-to-native is **4.123751737977231e-9** (923 samples). The mesh volume is **149.3552153206597**, with closed tessellation. All four unsupported companion cases retain unchanged inputs.

All **45** targeted archive members were byte-verified; archive SHA-256 `53d07e4e01d662899660fe62d7eaeb69439801c466dd156f9e69a7ffb418eed6`. Replay executable SHA-256 `f599adbd30a7c5b3e2c2aeebfba32ef4e9eebaa909a975830f042fd67aeb4f60`. Native input/reference bytes are verified against the 2026-10-13 closed-Bézier-import capture.
