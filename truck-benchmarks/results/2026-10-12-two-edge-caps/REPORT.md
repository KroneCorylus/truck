# Fillets on two-edge curved caps

Open fillet wires now retain the middle of a support boundary edge when both
contacts cut that same edge. Terminal faces are rebuilt after collecting both
end trims, allowing both ends to share a terminal face. Public APIs and
diagnostic meanings are unchanged.

Both regressions in `truck-shapeops/tests/two_edge_blend_caps.rs` fail at
`trim_faces` on parent 7f571cef. They now pass for a semicircular cylinder rim
and an obliquely swept parabolic rim equivalent to OCCT blend/simple/G1.
The tests check independently derived radius, normals and volume, topology,
closed meshes, STEP, input immutability, transformed/reversed selection and
two radii or scales. No existing test was changed. Derivations are in
`truck-shapeops/tests/TWO_EDGE_BLEND_CAPS.md`.

The full release shapeops run passes 330 tests, with zero failures and one
existing ignored test, across 63 targets. That run included the half-cylinder
regression; the subsequently added parabolic regression also passes in the
final two-test target run. There are 331 distinct passing tests across these
runs. All-target release Clippy for shapeops and benchmarks and the action
benchmark smoke check pass. No performance improvement is claimed.

The targeted original G1 capture now constructs with consistent geometry,
unchanged input, a closed Truck mesh and a native-valid single-solid STEP.
Its STEP volume is 132185649.97246. Independent integration converges to
132185649.98388772, with successive tight quadratures differing by 0.000424
in the original capture's volume units. The mesh volume is 132184965.46396536;
the regression separately checks the common chord-error allowance.

OCCT's result volume is 132189932.40065464. Maximum sampled distances are
0.98749869748551278 native-to-Truck (921 samples) and 0.018545749048056535
Truck-to-native (886 samples). These exceed the audit threshold of 0.001;
the case remains a shape discrepancy despite the repaired construction and
independent checks. The targeted replay used the working implementation before
commit; the artifact manifest records that scope explicitly. A clean full
capture is recorded separately after commit.

The archive contains both inputs/results and the Truck STEP and report. Its
manifest records every artifact hash. The independent integral script is
standalone and uses neither kernel. General support collapse, multiple cuts
that consume an entire shared boundary, global collision checks and other
curved corner failures remain outside this fix.
