# Isolated chamfers can consume short neighboring edges

The miter route rejects a chamfer if a neighboring boundary edge collapses or
reverses. The existing straight-edge solver can walk past such short segments
while retaining their support faces, but OutsideNeighbour prevented that solver
from being tried. Equal and unequal isolated-edge chamfers now try that existing
route after this rejection. The retry is limited to one selected edge; its own
boundary, topology and geometry checks remain in force. If it also rejects the
input, the original diagnostic is retained. No new geometry solver, fitting
parameter, public API or diagnostic code was introduced.

Both new regressions fail on parent ed603e7d12892bc434d335147ebbfd33687c3a4e with OutsideNeighbour
and pass unchanged after the fix. A 10×10×10 box receives a unit chamfer on one
vertical corner, then a second chamfer on a neighboring bottom edge. The second
cut consumes the short diagonal edge left by the first bevel without consuming
its supporting face. Six variants cover equal and unequal setbacks, scale,
rigid transforms and reversed face order. They verify the consumed edge is gone,
eight output faces, one new bevel, its independent plane equation and outward
normal, immutable input, analytic material and shared solid/robust STEP checks.
The single-edge convenience entry point is also checked against the same volume.
Existing oversized-neighbor and collision rejection tests pass unchanged.

For box width Lx, height H, first corner setback a, second planar inset b>a and
vertical setback c, the first cut removes a²H/2. The second wedge would remove
Lx*b*c/2, but its overlap with the first cut must not be subtracted twice:

    overlap = c * integral(0..a, (a-t)*(1-t/b) dt)
            = c * (a²/2 - a³/(6*b))
    remaining = box_volume - a²H/2 - Lx*b*c/2 + overlap.

The fixture uses a=1, b=2, c=1 or 2. Thus input volume is 995 and output volumes are
985.4166666666667 or 975.8333333333333, scaled cubically. The new plane satisfies
y-(b/c)z-(10-b)=0, with outward normal proportional to(0,1,-b/c).
These formulas are independent of both kernels' outputs.

Three fresh native OCCT controls construct the initial bevel, preserve shared
selection/face identity in a compound, and perform equal or unequal second cuts
at scales 1 and 0.2. Native inputs and outputs match analytic material and remain
valid single solids. Truck replays preserve input and export valid single-solid
STEP; maximum sampled boundary difference is 5.0242958677880805e-15, maximum analytic STEP
volume error is 1.1368683772161603e-13. Finite sampling is not a global bound.

Full shapeops release: 393 passed, 0 failed, 1 existing ignored.
All 39 replay tests, all-target release Clippy, single-threaded actions smoke and
final replay build pass. The old sequential_chamfers_reproduce_kernel_gap test expected the repaired
operation to fail. Its contract was rewritten before reapplying the fix: the same
four ordered axis-pair cases now require exact material, shared solid/STEP checks,
complete face history and convex geometry, while retaining input immutability.
The strengthened test fails on the parent with OutsideNeighbour. Its history
check accounts for unchanged IDs or one replacement, as the API documents; the
initial draft wrongly required every face to appear in modified_faces, and that
failed check is retained in initial-sequential-history-check.txt. The initial
full-suite failure is retained in initial-shapeops-tests.txt; all remaining tests
and rejection guards are unchanged.
Source/executable hashes are in metadata.json. No performance claim is made.
Archive: 68 byte-verified members, SHA256 598d8c39f1a3cb4bd5655bf609fbdac768f2ddd2baf6559738237cac7f5f31ac.
A clean full capture follows this implementation/proof commit.
No consumer/pin changes, push or PR. Multi-edge consumption, removed support faces
and general collisions remain outside this routing change.
