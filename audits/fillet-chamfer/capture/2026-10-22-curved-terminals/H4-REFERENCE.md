# H4: native polar trim is not a reliable solid reference

H4 rounds the concave axis of a 270-degree ellipsoid/conical sector. Its
radius 10 cylinder has center (10,-10) in the transverse plane. This operation
fills the reentrant corner. The analytic ellipsoid cap retains its pole at
(0,0,100), and independent area/height integration gives volume
13159710.462164925. The original input volume is13155419.23690726.

The native captured result reports valid, but its volume is13151908.604695952
and its boundary misses the pole by about 4.14211. Its revolved face trim
extends past one full angular period and stops below the meridian pole.
Truck preserves the pole; the existing regression verifies its topology,
rolling radius, material integral and STEP vertex loop.

A separate read-only check samples nine analytically interior points near the
pole. OCCT classifies all nine as IN in Truck's STEP output, but ON in its
native result. Their computed native-boundary distances range from about
1.48849 to 4.28356, well above the 1e-7 classification tolerance. Six points
are IN in the original input; three lie in the analytically filled corner.
The native validator accepts input and both results. These contradictory
classification/distance observations show that validator success alone does
not establish a correct native polar trim.

Fresh procedural native controls use the capture's exact fillet settings and
the same normalized profile at scales 0.5,1,2,10 and100. All inputs are valid
and remain unchanged. Results at the three smallest scales are invalid;
the other two are valid but give different normalized material and pole
behavior. The scale 100 rebuild differs slightly from the captured construction
in floating-point placement, so its numbers are reported separately.

These findings establish a native-reference quality problem; they are not
permission to remove Truck's independently verified polar material. The
original sampled-comparison failure and corpus status remain unchanged.
See `reference-evidence.json` and `reference-evidence.zip` for exact programs,
commands, raw shapes, classifier states, native library hashes and outputs.
