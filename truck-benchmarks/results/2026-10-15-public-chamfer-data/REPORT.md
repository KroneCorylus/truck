# Public chamfer data and explicit external fixture support

Both audit runners accept additional data directories while preserving OCCT/data
as the first search path. They record external file hashes and recheck them before
marking the run verified. The catalog validates that marker and exact raw bytes,
either on disk or in the evidence archive. Passing dedicated chamfer grids are
selected even when Tcl helpers construct command names dynamically.

Four new parent-failing regressions cover directory/path/hash behavior, missing
directories, dedicated-grid selection and catalog rejection of unverified/changed
or missing archived data. All 22 audit and nine capture tests pass. No Rust,
native plugin, kernel geometry, public API or consumer changes. Prior Rust tests,
Clippy and benchmark smoke were not rerun for this Python-only change.

The public dataset at https://github.com/a-betenev/opencascade-dataset, pinned
revision d78a7a9e6de57699261fa03641ec1b5bafac39a1, supplies 32 of 36 required names.
All 535523 bytes match Git blob hashes and sizes. Native execution of 279 unchanged
cases gives 239 passes, 22 known failures, 18 remaining missing-data skips. Native
run and fixture archive: audits/fillet-chamfer/runs/2026-10-15-public-chamfer-data.
No cases were synthesized or relabeled as passing from file availability alone.

Targeted capture of four passing native entries produces 11 operation records:
4 constructions, 5 operation failures and 2 explicit unsupported curved-reference
intents. All native entries pass. Two constructions pass sampled comparisons at
0.001; two retain measured discrepancies (equal_dist/A2 about 0.00133 and sequence
A4/001 about 0.02121). All four exports are native-valid single solids. These are
independent per-operation replays of native inputs, not an end-to-end Truck sequence.
No full-parity claim. Full clean capture follows this implementation/proof commit.

Targeted archive: 159 byte-verified members, SHA256 a94364d34281a354c113942e6ed8829e46f65def323762ccbb7067fabde1e9fa.
Metadata retains source hashes, provenance and status counts. No speed claim.
