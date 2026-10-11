# Spherical meridian input

`input.step` is the byte-for-byte input captured before the radius-10 fillet in
OCCT V7_9_3 `tests/blend/simple/D4`. It is a radius-150 sphere between latitudes
-60 and +60 degrees, restricted to a 90-degree azimuthal sector and rotated 90
degrees about Z. The selected meridian lies in Y=0, X<0.

The source task, original BREP, physical selection, native result and SHA-256
hashes are in `audits/fillet-chamfer/capture/2026-10-11-support-domains/`.
The fixture corresponds to `blend/simple/D4/capture/001.input.step` in its archive.
Independent input volume is pi/2 * (R^2*H - H^3/3), with R=150 and H=R*sqrt(3)/2.
