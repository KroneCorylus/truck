# Bounded STEP edges retain their trimmed domains

The importer now cuts bounded 2D/3D edge curves at their endpoint vertices,
respecting edge sense. Polynomial/rational splines keep their original parameter
chart and derivatives; polyline endpoint lookup follows its changed chart after
cutting. Complete closed curves retain both endpoint branches. Unmatched points,
nonfinite/out-of-domain parameters and reversed intervals reject explicitly.
No modeling tolerance, surface geometry, public API, dependency or consumer changes.

Three focused tests fail on the parent; a full-closed-curve control passes there.
The native C1 solid regression also fails parent geometric consistency. After the
fix, all four importer tests and the solid's original/transformed topology,
material, closed mesh and STEP round-trip checks pass. Full affected release
suites: 498 passed, zero failed, one pre-existing ignored. All-target release
Clippy (stepio, shapeops, benchmarks), action benchmark smoke, 15 replay tests and
the release replay build pass. No speed claim; no tolerance/assertion was loosened.

The unchanged native fixture is documented in
`truck-shapeops/tests/fixtures/bounded-step-edges/README.md`. The independent
volume oracle is 0.002952545064816 from adaptive Gauss–Kronrod integration;
STEP convergence from requested 1e-8 through 1e-12 varies by under 6e-15 and
original BREP differs by 1.53e-10. Ordinary native integration is unstable on
this high-degree geometry. `native-input.txt` preserves the initial comparison
against that unsuitable ordinary estimate; `native-input-adaptive.txt` uses the
corrected reference and reports a valid single solid with error 1.6e-15. These
are numerical convergence observations, not certified error bounds.

Build the independent volume probe from the repository root:

```sh
g++ -std=c++17 -O2 -Wall -Wextra -Wpedantic \
  -isystem target/occt-sdk/usr/include/opencascade \
  truck-benchmarks/results/2026-10-15-bounded-step-edges/native-volume.cpp \
  -Ltarget/occt-sdk/usr/lib64 -lTKDESTEP -lTKXSBase -lTKTopAlgo -lTKBRep \
  -lTKGeomBase -lTKG3d -lTKG2d -lTKMath -lTKernel -o target/bounded-edge-volume
LD_LIBRARY_PATH=target/occt-sdk/usr/lib64 target/bounded-edge-volume \
  truck-shapeops/tests/fixtures/bounded-step-edges/C1.step
```

Three unchanged native C1 sequence entries pass and yield 24 targeted captures.
Four previously inconsistent inputs now reach operation handling: equal-distance
steps 002/003 reject with preserved input (trim_faces/validate_contact), while
two-distance steps 002/003 explicitly lack captured curved reference support.
There are no added successful chamfer constructions in this group. Overall:
three operation failures, four unsupported intents, nine inconsistent inputs,
eight import failures. Seven angle-mode inputs now reject unmatched bounded
vertices; equal-distance step006 reaches unsupported modeling conversion.
All eight were already inconsistent on the parent.

Residual inspection of equal-distance step004 finds correct endpoints but a
sampled curve/support gap of 1.4349e-6. Angle step002 has vertex/curve gaps of
8.5e-6 through 6.5e-5, with some nearest parameters outside the spline domain.
Keep these explicit input fidelity gaps; do not increase global tolerance or
claim all21 C1 inputs are repaired. Read-only inspector sources and outputs are
included. The build helper resolves the replay's exact Cargo dependency graph.

The targeted archive contains 228 byte-verified members, SHA256
`2a86d0be667c264357111d9f1886a0807780d7afb6b990e0022ff9cc849436f2`.
Source hashes and validation counts are in metadata.json. Full clean corpus
capture follows the implementation/proof commit. Native tangent propagation,
closed-shell imports, curved reference support, missing fixtures and prior
sampled discrepancies remain open. Sequences replay native inputs independently.
