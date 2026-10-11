# C7 cone export after the domain fix

This fresh physical-selection replay uses the STEP preparation implementation
committed in `fa1a142d`. The recorded kernel HEAD is `d7467cdf` because validation
preceded the commit; the manifest records the dirty worktree and actual source
hashes. Its `truck-stepio/src/out/modeling.rs` hash matches the committed fix.

The native instrumented test stays OK. Truck returns consistent geometry, preserves
input, produces a closed mesh and exports a valid native single-solid STEP. Maximum
bidirectional sampled distances are 1.7632e-6 and 2.7516e-6; the prior baseline has
a 4.2283-unit forward discrepancy. Volumes are 76158.86301431 (native) and
76158.86323932 (Truck STEP), versus the independent integral 76158.86301257.

No full parity verdict is assigned. These checks cover this physical task and its
sampled output geometry, not global intersections, editing or all curved ends.
Kernel validation and broader transformed/reversed cases are documented in
`truck-benchmarks/results/2026-10-11-curved-fillet-step-domain/REPORT.md`.

The earlier baseline remains available separately. Every listed artifact and the
archive itself is hashed; capture and replay source, kernel sources, executables,
runtime, native harness and fixture hashes are recorded in the manifest.
