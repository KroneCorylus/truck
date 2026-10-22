# G1: native parabolic blend differs from the analytic rolling envelope

The independent construction and material equations are documented in
`truck-shapeops/tests/TWO_EDGE_BLEND_CAPS.md`. At captured scale 1000 the exact
rolling radius is1000, and the independent volume is 132185649.98388772.

A new read-only native probe samples 1,411 points on the generated B-spline
blend face and its boundary edges. For each point it inverts the analytic
x/y section coordinates and compares the z coordinate and distance to the
corresponding rolling center. This is a coordinate-conditioned envelope
residual, not a closest-distance or global-error bound.

The native result has maximum z residual 1.346997734 and rolling-radius
residual 0.143436880. Truck's exported result has respective sampled residuals
0.0003193793 and 0.0000044168. The captured native volume is 132189932.39942582,
about 4,282.416 above the independent integral. Native stored edge tolerance
is about0.000005845; the discrepancy exceeds both that and the 0.001 audit
threshold. The independent volume convergence and direct radius/normal kernel
controls predate this diagnosis and remain unchanged.

This evidence identifies a native-reference mismatch with the specified
rolling envelope. It does not justify making Truck reproduce that mismatch.
The recorded native/Truck comparison still fails and its status is unchanged.
All measured input/result shapes remain byte-identical after probing.
See `reference-evidence.json` and `reference-evidence.zip` for programs,
commands, raw shapes, outputs and hashes.
