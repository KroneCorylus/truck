# High-valence spherical fillets: validation

Parent: `d0c7a3bf` (2026-10-11 working session). This is a capability and correctness
change, with no speedup claim. No operation timings were collected; the existing action
benchmark suite ran in smoke mode with one Rayon worker.

The new regression first fails on the parent with `UnsupportedTopology` for a regular
four-sided pyramid at radius 0.2. The implementation now solves the common inward-offset
center using all supporting planes and checks every residual before constructing exact
spherical/cylindrical contacts. It adds no dependencies, unsafe code or public API.

Validation:

- `cargo test -p truck-shapeops`: **305 passed, 0 failed, 1 existing ignored**, including
  seven doctests. The new cone probe was started separately after this suite had compiled
  and is not included in these totals or this change.
- `high_valence_fillets`: 16 successful regular-pyramid configurations (4/5/6/8 ridges,
  two radii, original/rigid transform), with independent analytic volume, geometry,
  topology, closed mesh, sphere dimensions, G1 joins, face-history counts, selection
  reversal, immutable input and STEP. Two cases also cut through the spherical corner
  and validate cap volume and STEP. Incompatible offsets and excessive radius reject
  without mutation.
- `fillet_edges::all_pyramid_edges_with_four_edge_spherical_vertex`: all eight edges of
  a square pyramid, checked with a separate Steiner-volume formula, smooth contacts,
  topology, closed mesh, STEP and retained original-input checks. The old rejection
  expectation changed because the new construction supports this case. The full suite
  executed the updated test; its result is also recorded separately.
- `cargo clippy -p truck-shapeops --all-targets -- -D warnings`: passed. The final changed
  test targets and library passed Clippy again after the all-edge contract update.
- `RAYON_NUM_THREADS=1 cargo bench -p truck-benchmarks --bench actions -- --test`: passed.
- Targeted nightly rustfmt check and `git diff --check`: passed.

`parent-regression.txt` records the initial failing regression before construction
changed. `sources.txt` records relevant source hashes. Raw validation logs accompany
this report. App code is unchanged and app tests were not rerun for this batch.

The paired OCCT probe and raw observations are in `audits/fillet-chamfer/probes`.
OCCT 7.9.3 also succeeds with valid single solids in all eight native-coordinate cases,
but constructs non-spherical corner patches with different volumes. No claim of identical
corner geometry or general high-valence parity is made. Partially selected corners,
inconsistent offset planes, unequal radii and general curved filling remain open.
