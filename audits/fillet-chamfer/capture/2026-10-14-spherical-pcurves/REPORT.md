# Full capture after spherical parameter-curve import

Frozen implementation/proof revision: `dbb54343d06c306c29849b15271f46f40f789c54`. Clean tracked checkout at capture
start; kernel, capture-source and executable hashes match before and after.
All 156 native entries passed, producing 161 captures.

- 105 valid constructions, consistent geometry, unchanged inputs,
  closed meshes, prepared STEP and native-valid single-solid exports.
- 96 combined limited comparison passes at 0.001.
- 53 operation failures, 2 import failures, 0 unhandled intents, 1 incomplete.
- Zero panics/timeouts; no previously passing check regressed.
- All 104 previous successful result/comparison records are exactly unchanged.

Only draft/angle/F1 changes status, from input-import-failed to operation-succeeded.
The hemisphere/box union and radius-2 fillet now import and construct. The separate
proof directory derives input and result material from dimensions and checks the
export using the native STEP reader/integrator. Native comparisons are sampled,
not certified global distance bounds. The later draft in F1 is not captured.

The two remaining imports are blend/complex/B5 and blend/simple/J7. Each contains
one CLOSED_SHELL, no MANIFOLD_SOLID_BREP, and native input metadata reports zero
solids. This group does not reinterpret their representation as a solid.

`comparison-summary.json` lists every status/check/numeric difference.
`results.json` records commands, source revision, timeouts and artifact hashes.
`artifacts.zip` is byte-verified against the raw output and manifest, including
runtime.log. Archive SHA-256: `68d70320f77f9af4a121e5ee87a556c3d3e02abb6964d78fea3568e395038120`.
Raw output: `target/occt-capture-spherical-pcurves`.

Catalog generation/check and the 19 audit plus 2 capture tests are recorded in
`catalog-checks.txt`. No overall parity percentage or performance claim is made.
