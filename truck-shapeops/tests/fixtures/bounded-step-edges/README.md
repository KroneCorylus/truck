# Native bounded spline edge input

`C1.step` is an unchanged 141861-byte copy of operation 002's input
from native `chamfer/equal_dist_sequence/C1`, OCCT 7.9.3, captured in
`audits/fillet-chamfer/capture/2026-10-15-public-chamfer-data/artifacts.zip` as
`chamfer/equal_dist_sequence/C1/capture/002.input.step`.
SHA256: `973bde800fdc1692c38a5b746a3d2338698de7af7c94eaa8f98764a4f846e581`.

The source `CCH_001_qhff.rle` is from the public opencascade-dataset revision
`d78a7a9e6de57699261fa03641ec1b5bafac39a1`; complete source provenance and raw
bytes are retained in `audits/fillet-chamfer/runs/2026-10-15-public-chamfer-data`.
This input follows the first native chamfer. Four bounded spline edges have
interior endpoint vertices; importing their full spline domains causes endpoint
errors between 0.010 and 0.0275 and fails geometric consistency.

The independent material reference is 0.002952545064816, from native adaptive
Gauss–Kronrod volume integration of this STEP. Values converge to within 6e-15
between requested accuracies 1e-8 and 1e-12. Original BREP integration differs by
1.53e-10. Ordinary native integration is unstable for this high-degree source
geometry, giving 0.002960726849837509 in capture metadata; that value is not the
regression oracle. Both methods and convergence logs are preserved in
`truck-benchmarks/results/2026-10-15-bounded-step-edges`.

The regression uses the shared solid/STEP harness at 1e-5 in original and rigidly
transformed frames. This is an independently integrated arbitrary source solid,
not an analytic primitive. No imported topology, source geometry or test tolerance
is modified to make the fixture pass.
