# Preserve supported fillet replay export fitting

The full clean capture of 2346b975 found 13 previously successful fillet exports
with `step_prepared=false`. Constructed geometry, input preservation and meshes
still passed. The new globally applied 1e-7 STEP fit exceeded the exporter's
subdivision budget for those procedural fillet surfaces. This is an audit replay
regression, not a newly unsupported fillet construction.

Replay now retains 5e-5 for fillet STEP fitting and uses 1e-7 for chamfers. Each
structured result records its actual `step_fitting_tolerance`. The general
exporter and kernel sources are unchanged. Existing coarse fillet fits follow
the documented robust STEP tessellation contract; chamfers keep the strict A4
round-trip check introduced with complete native contour capture.

The new native D4 regression fails on parent 2346b975f9760071ee6b71f6efa2d354dbd698e2 at
`step_prepared`. It passes with the correction, including immutable input,
closed constructed mesh, consistent geometry, restored STEP preparation,
imported topology and the shared robust STEP round-trip/material harness.
All 25 replay tests pass, as do all-target release Clippy, action smoke and the
final replay build. Kernel code is unchanged from its 504-pass validation.

Targeted native proof: D4's result (excluding added metadata) and its complete
native STEP comparison are EXACTLY restored to the preceding stable corpus.
A4/001 still captures/replays all three native contour edges, returns a valid
single-solid STEP, and has sampled maximum boundary difference
2.229100205894138e-8. Both native cases pass. Remaining A4 sequence results are
retained separately; no all-sequence or global-distance claim is made.

The initial regressed full run is preserved in initial-full-capture.zip with
7676 byte-verified members, SHA256
bc042ec67c48bcce23dc0d08ff03f971decba2270c0a9876e77df1fc60d2eed9.
Its comparison records all 13 unchanged-intent regressions, 58 corrected chamfer
scopes, unchanged raw seeds and unchanged native reference properties. It must
not be treated as the stable checkpoint.

Corrected targeted proof archive: 134 byte-verified members, SHA256
0623341212d13bd940b0086edd5b2a903ad0ccd2717b1a0705d47344b3299532. Source and executable hashes are recorded in metadata.json.
A fresh clean full capture follows this correction commit, compared against the
preceding stable two-edge-face checkpoint. It must preserve all successful checks
for unchanged intent while documenting corrected complete-contour outcomes.
No existing test was loosened. No performance or full-parity claim.
