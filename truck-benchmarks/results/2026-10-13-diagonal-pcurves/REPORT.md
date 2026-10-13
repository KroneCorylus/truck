# Exact diagonal STEP pcurves and safe rolling end trims

The second OCCT `blend/complex/G9` input now converts to modeling geometry.
Linear pcurves on cylinders, cones and tori retain their exact parameterization
through the existing modeling pcurve representation. Cylinder/cone revolutions
keep their processor placement and orientation; torus revolutions explicitly
swap the major/minor coordinates. Existing generator lines and isoparametric
torus circles retain their compact representations. Non-finite and collapsed
trims are rejected.

The curve regressions compare points and first/second derivatives through
periodic seams, reversed curves, reversed supports, general affine transforms
and reflections. The old diagonal-rejection tests now assert successful exact
conversion and derivatives; the previous roundoff bounds and invalid-input
checks are retained. A large transform that prevents flattening a near-seam
trim now uses the exact pcurve instead.

The unchanged native rounded-offset-cylinder fixture passes both preprocessing
modes, with and without rigid motion. Its regression retains the raw-solid
checks and adds modeling consistency, closed topology/tessellation, independent
volume and STEP round trips using the shared solid/blend harnesses. The expected
volume remains approximately 6965.56812665, from the independent disk-overlap
integral documented beside the fixture.

Reaching the previously inaccessible fillet solver exposed an unchecked rolling
contact during an end-trim Newton search. That search now checks contact
existence at each iterate and returns failure when the contact disappears.
The regression requires `BlendConstructionFailed` without input mutation in
both wire directions and all four import variants. The final targeted native
replay reaches `blend`, selects one edge and returns `operation-failed` with
`input_unchanged: true`. The radius-4.9 fillet itself remains unsupported.

## Evidence and validation

- Parent `4a508949`: the new curve tests fail on unsupported diagonals and an
  accepted collapsed cylindrical trim; the native-solid test fails at modeling
  conversion. See `parent-pcurves-all.txt` and `parent-g9.txt`.
- With conversion alone, the native operation and expanded solid regression
  reproduce the rolling-contact panic. See `g9-panic-backtrace.txt` and
  `unguarded-g9.txt`. These are intermediate failure evidence, not final results.
- Final release tests for `truck-stepio` and `truck-shapeops`: **447 passed,
  0 failed, 1 existing ignored**, across 84 test/doc targets.
- Final all-target release Clippy for stepio, shapeops and benchmarks: passed.
- Action benchmark smoke with one Rayon worker: passed.
- Final production replay build and targeted G9 replay: passed without panic;
  the operation returns the diagnostic described above.
- Touched Rust files were formatted individually. No existing success assertion
  was removed, weakened or ignored.

Final command lines are recorded in their logs. `metadata.json` contains source,
fixture and executable hashes; `replay-inputs.json` verifies the targeted input
and native reference against the committed capture archive. Logs prefixed
`conversion-only-` predate the end-trim guard. No performance claim is made.

The full capture has not been rerun for this change. The historical full run
remains unchanged; this targeted input recovery does not establish fillet parity.
