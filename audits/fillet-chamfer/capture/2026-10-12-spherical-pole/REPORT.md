# Captured replay after spherical-pole fillet support

The full run at clean 3f16afc9 (kernel change 6db16d09) passes all 156
native entries. Its 161 captures contain 74 constructed Truck results,
30 construction failures, 25 import failures, 23 inconsistent inputs,
eight unhandled intents and one incomplete capture. There are no timeouts
or panics.

E3 is the sole construction-status change from the preceding spline-rim
capture. All 74 constructed results preserve input and geometry consistency,
have closed Truck meshes, prepare STEP and yield native-valid single solids.
Seventy-two meet the combined limited checks, including sampled boundary
distances in both directions no greater than 0.001. G1 and tolerance-grid
A1 retain their previously recorded shape discrepancies.

The radius-10 spherical meridian now reaches its polar end face. E3's
native-to-Truck maximum sampled distance is 0.0001213415038449 at 917
stations; Truck-to-native is 0.0000218593241064 at 915 stations. Truck STEP
volume is 3479486.1878489405, compared with native 3479486.2282060464 and
independent integrated volume 3479486.1779266507. The closed Truck mesh
volume is 3479455.724002516.

Parent-failing solid regressions cover the original scale and two smaller
scales, rigid motion and reversed selection. They check radius/normals,
independent material volume, consistent closed topology/meshes, STEP and
unchanged inputs. Primitive and export regressions isolate signed polar
contacts, revolution projection and concentrated STEP fitting error. The
five-crate release run records 879 passes, one unchanged PCurve property
failure and five existing ignored tests; the final affected shapeops rerun
has 338 passes, no failures and one existing ignored test. Clippy, action
smoke and catalog tests pass. See
`truck-benchmarks/results/2026-10-12-spherical-pole/REPORT.md` for the
independent volume derivation, failure proofs, validation logs and timings.

All case artifact hashes were verified before archiving. The manifest
retains clean source, replay binary, native runtime and harness provenance.
These sampled comparisons and targeted regressions remain limited evidence;
other construction/import gaps and full feature-parity verdicts stay open.
