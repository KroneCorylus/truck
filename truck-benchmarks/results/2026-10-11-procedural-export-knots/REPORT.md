# Smooth longitudinal fillets and spherical-meridian STEP export

The captured OCCT `blend/simple/D4` task constructs a radius-10 fillet on the
meridian of a radius-150 spherical sector. The parent builds consistent topology
but cannot prepare the result for STEP at 0.00005 tolerance. Its fillet has
longitudinal derivative jumps: a recorded first derivative changes from
(-137.460108, 0.029929, 58.717925) to (-137.208531, 0.029820, 58.610457) across
an internal knot. See `smoothness-parent-regression.txt`.

The fillet's section frame differentiates its fitted contact curve once.
Piecewise quadratic contact interpolation leaves the frame/fillet only C0 at
simple internal knots. The intersection's projection plane follows a leader
derivative, producing position jumps up to about 0.000183 in the captured trim.
That exceeds twice the export tolerance, so splitting the exporter cannot produce
a continuous curve within tolerance. `position-jump-diagnostic.txt` records the
one-sided samples. No exporter production change is retained.

The contact interpolator now uses degree four when internal knots are necessary.
Standard averaged interpolation knots retain C3 contact curves and C2 section
frames on smooth supports. Three/four samples use a single quadratic/cubic Bezier
span with no internal knots. Averaging relative to the first station avoids
needless loss of precision in shifted domains. Adaptive sampling, contact checks,
fitting tolerances, topology construction and STEP tolerances are unchanged.

The native-fixture regression uses both the original task and a rigid transform
with reversed selection. It checks first/second derivative limits at internal
knots, toroidal radius and normals, geometric consistency, input immutability,
closed topology/meshes, independent volume and STEP round trips. The volume is
computed by integrating the removed circular strips through the capped spherical
sector, independently of the kernel's generated surface or tessellation.

## Timings

`spherical_meridian_export` loads/preprocesses the same fixture outside timing,
warms up once, then records five construction and STEP-preparation samples with
`RAYON_NUM_THREADS=1`. It checks geometry consistency after timing. Parent and
current builds differ only in the production contact-interpolation change.

Construction median is 20.006 ms before and 17.626 ms after; longitudinal knot
count drops from 56 to 32. All five parent STEP preparations fail (33.765 ms
median); all five new preparations succeed (134.153 ms median). Failed and
successful preparation times measure different outcomes, not an export speedup.
Exact samples are in `before.json` and `after.json`.

## Validation and scope

Parent failures, focused round-trip results, full affected release-suite output,
all-target Clippy and action-smoke output are retained here. Full release tests
across geometry, modeling, meshalgo, shapeops and STEP report 852 passed, one
pre-existing failure and five existing ignored tests across 126 target results.
Clippy and the action smoke pass. The existing PCurve
polynomial property failure is documented and reproduced on the preceding parent
in `../2026-10-11-spherical-meridian-step/REPORT.md`; its assertions remain unchanged.
Native replay comparisons are recorded separately in the audit. This repairs the
interpolation-induced seams on smooth supports; it does not certify arbitrary
support singularities, global collision freedom or full OCCT parity.
