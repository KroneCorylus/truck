# Final native fillet contours and radius assignments

Fillet capture now records complete contours and their final constant radius
after all native Add calls. Repeated seeds can replace radii; different radii on
separate seeds can create a nonconstant law. Null constant_radius identifies
that unsupported law, retaining every raw seed and contour member.

A native probe demonstrates why capture must use the whole-contour query:
before Build, unassigned propagated edges can report zero per-edge radius even
when IsConstant(contour)/Radius(contour) correctly report a positive constant.
The probe and both native control runs are archived.

The fresh-process DRAW wrapper observes successful tolblend/continuityblend
calls with native numeric conversion and forwards the original scope, return
value and error options. It starts from the pinned native defaults before test
setup. Contour discovery receives exact observed settings. Observation failure
invalidates further capture. R/Q/P representation is captured; replay supports
rational C1 constant positive contours and explicitly rejects variable laws,
other representations and C0/C2 approximation continuity. Truck retains its own
fixed modeling/export tolerances; solver approximation-setting parity is not
claimed. Legacy schema-1 input remains explicitly seed-only.

Replay validates complete connected contours, unique ownership, seed membership,
final radius assignments and required metadata before construction. It uses the
existing multi-edge and wire APIs atomically, with no partial seed fallback.
No kernel source or public API changed. No existing test was loosened.

Validation: 33 Rust replay tests pass, including six new tests. Five fail on
parent 6b07572a15221d8ec45eaf3e5b647b69f05cbbae; the invalid-schema test already rejected the
request on that parent. Thirteen capture tests pass; two new settings tests and
one summary test fail before their corresponding implementation changes.
All-target release Clippy, action smoke, plugin build and final replay build pass.
The previous 504-pass kernel validation was not repeated or represented as new.

New A4 tests exercise complete single, replaced and zero-then-valid contours in
both traversal directions. Constructed geometry is checked by the shared solid
closure/material harness, and exported fitted fillets by the existing robust
STEP harness. The first new test incorrectly required ordinary tessellation of
coarse fitted STEP; that failure is preserved. The corrected test explicitly
checks constructed-solid closure/material plus the documented robust export
contract, at unchanged tolerances. Existing tests and harnesses are unchanged.
The box replacement control has independent analytic material
6000 − (1 − pi/4) × 3² × 30. A4 material values are native reference checks.

Eleven native controls build valid single solids. Replay constructs seven
complete requests and explicitly rejects four unsupported intents. All seven
export valid native-checked single-solid STEP and pass sampled comparisons at
0.001. Maximum A4 sampled difference is 6.671012283394402e-6; the box difference
is 1.0223701381680905e-14. Controls include repeated and different seeds, restoration
to one constant radius, R/Q/P flags, exact changed tolerances and C2 continuity,
and zero followed by a valid seed assignment.

The initial control check compared a restored BREP export to pre-read bytes;
OCCT normalizes direction components at roundoff when reading. The preserved
failure led to the correct in-memory baseline: all A4 input exports and the
post-control export are exactly byte-identical. Box input bytes are unchanged.
Truck replay independently verifies immutable inputs.

Archive: 222 byte-verified members, SHA256 68965cc343a4b388d9cbb3ed3a8fe86733c7089d9ea67ac8e0bd5d2c469bd995.
Source, fixture, executable and library hashes are recorded in metadata.json.
A clean full capture follows this implementation commit. Corrected fillet scope
must be distinguished from unchanged-intent regressions. Finite sampled checks
do not establish global distance, full parity, continuity/history or latency.
