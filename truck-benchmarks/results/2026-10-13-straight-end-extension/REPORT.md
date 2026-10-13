# Straight end-boundary extension for the E9 torus fillet

E9 rounds the meridian of a torus sector with major radius 200, minor radius
50, meridian angle 160 degrees and revolution angle 90 degrees. The fillet
radius is 10. At the reentrant end, its plane-side contact meets the horizontal
end plane beyond the original straight boundary's vertex. The old bounded
intersection correctly found no intersection inside that edge, but the blend
could not extend the straight support to the required contact.

The new path first attempts the existing bounded trim. If that fails at an
open chain end, it can recognize an exact line and extend only its endpoint
next to the chain. The opposite endpoint, curved edges, interior seams and
closed chains keep their original domains. Contact evaluations remain bounded.
Both incident faces receive the same replacement edge through the existing cut
map. The ordinary geometry and planar boundary checks remain in force.

## Independent geometry

`independent-volume.py` evaluates the rolling sphere's envelope directly from
the torus and plane equations. It does not read Truck or native output.
Write R=200, r=50, f=10, theta for meridian angle, and h=r sin(160 degrees).
The center is

```
rho = R + (r-f) cos(theta)
C = (sqrt(rho^2-f^2), f, (r-f) sin(theta))
N = (cos(theta) C.x/rho, cos(theta) f/rho, sin(theta))
P = (0,-1,0)
d = N dot P
U = (P-dN)/sqrt(1-d^2)
B(theta,w) = C + f [N cos(beta) + U sin(beta)]
beta = w acos(d),  0 <= w <= 1
```

Each beta section spans the two tangent sphere contacts. The envelope is
trimmed at z=0 and z=h; its upper theta endpoint is solved independently for
each w on the descending meridian branch. The divergence field (0,y,0) has
unit divergence. Its flux through both horizontal caps and the y=0 face is
zero, so the volume change is just the new blend flux minus the removed
torus-strip flux. The strip extends from azimuth 0 to
`asin(f / (R+(r-f)cos(theta)))`, with theta from 0 to 160 degrees.

Gauss–Legendre quadrature with 16, 32, 64 and 128 nodes per direction gives
the same volume **1537066.4574334237**. Complex-step derivatives evaluate the
envelope's area vector. `volume.txt` retains the convergence results, original
volume and both surface-flux contributions.

The regressions check radius and outward normals independently: for each
sampled surface point Q and outward unit normal n, the center Q-f n must
satisfy y=f and `(hypot(x,y)-R)^2+z^2=(r-f)^2`. They also check the analytic
extended contact position, unchanged input, generated-face count, geometry,
topology, mesh closure, independent volume and STEP round trip. Cases include
original scale, small scale, rigid motion and reversed selection. Unit tests
cover both edge orientations and contact ends, wrong-end rejection, curved
edge rejection and unchanged contacts on failure.

Both final E9 regressions fail at `trim_faces` against the parent production
source in `parent-final-regressions.txt`. Existing tests and assertions are
unchanged; no test was newly ignored.

## API scope

The generic chain fillet functions now require `C: TryInto<Line<Point3>>`.
The conversion must succeed only for exact lines; `truck_modeling::Curve`
already supplies it. This is documented in the Breaking changelog entry.
PiezaCAD calls the unchanged modeling-solid wrapper in
`cad-core/src/kernel/blend.rs:199`, so it needs no call-site migration.
Only explicit line geometry is extended; spline representations of straight
edges are not recognized by this conversion.

## Native replay

The original captured E9 operation now constructs a geometrically consistent
solid with unchanged input and a closed Truck mesh. Standard native STEP
reading accepts one valid solid. The ordinary native volume estimate for the
export is **1537066.4526273177**; Gauss–Kronrod integration gives
**1537066.4526492232**, an absolute difference of **0.0047842006** from the
independent integral. Maximum native edge/vertex tolerance is
**0.000008234261041079063**. No additional shape repair was applied.

The captured native reference volume is **1537066.4819177703**. Maximum sampled
distances are **0.000041647039422256268** and **0.000041645579725741633** in the
two directions, within the comparator's 0.001 threshold. This establishes the
limited sampled comparison for E9, not general fillet/chamfer parity.

The Gauss–Kronrod diagnostic reuses the committed
`../2026-10-13-polar-cap-seams/inspect-polar-step.cpp`; run it with the exported
STEP path and expected volume `1537066.4574334237`. Native input and reference
bytes come unchanged from the `2026-10-13-polar-cap-seams` capture archive.

## Validation

The complete shapeops release suite passed: **358 passed, 0 failed and 1
existing ignored test**, across 69 test/doc targets. All-target release Clippy
for shapeops and benchmarks, the single-threaded action benchmark smoke and
the final replay build passed. Commands and source/executable hashes are in
`metadata.json`; all nine replay archive members were byte-verified, including
the unchanged native artifacts. Source and replay hashes were unchanged after
validation. No performance claim is made.

The full native capture follows the production commit and is archived
separately. The earlier known PCurve derivative precision failure belongs to
the unchanged geometry crate; it was reproduced and recorded in the preceding
cubic-endpoint batch, and its assertion remains unchanged.
