# Spherical-pole fillets and local STEP fitting

OCCT E3 rounds a meridian of a radius-150 sphere quadrant with radius 10.
Its guide ends at the south pole, but the rolling contacts are regular.
Initializing directly at the singular guide chart failed to find the end
contact. The solver now retries from nearby regular sections and uses those
contacts to initialize adjacent-edge crossings, preserving signed-radius
selection. Along-wire fitting continues contacts through polar endpoints
with smaller stations so Newton does not change parameter branches.

The original-size input also exposed a projected-revolution Newton seed at
a pole, where the radial derivative is undefined. Projection now selects a
regular midpoint seed when the selected derivative is nonfinite. Exact
inclusion still verifies the recovered point; off-surface points remain
rejected. This restores the closed mesh of the unmodified input.

Finally, uniform STEP fitting could not resolve the concentrated endpoint
error. The fallback now interpolates the four homogeneous section controls
locally, retains source derivative breaks, and subdivides only spans that
miss the unchanged tolerance. It retains the 193-control-point budget and
shared cubic endpoints with legal interior knot multiplicities. The normal
uniform fitting path is unchanged.

## Validation

All three solid regressions fail on the previous modeling source and pass
with the fixes: scales 0.02, 0.04 and 1, including rigid motion and reversed
selection. They check rolling radius and normals, geometry consistency,
closed topology and tessellation, independent material volume, STEP round
trips and unchanged inputs. The independent volume derivation and Simpson
convergence are retained beside the STEP fixture.

Separate parent-failing regressions check positive/negative polar contact
radii with both support orders, exact and nearest revolution projection
near a pole, and a stored procedural fillet that the parent STEP fitter
cannot export. The export regression checks a dense parameter grid at
0.00005/sqrt(3), together with the control budget and knot multiplicities.
The stored surface isolates the export change from construction changes.

Five-crate release validation: **879 passed**, one pre-existing PCurve
derivative property failure, five existing ignored tests, 137 targets.
The unchanged PCurve parent failure and replay seed remain in
`../2026-10-11-spherical-meridian-step/`. All-target release Clippy and the
benchmark action smoke pass. An initial timing run exposed unnecessary pole
projections on closed wires. Restricting the new endpoint check to open
chains removes that overhead; closed seams retain their prior continuation
path. The complete affected shapeops suite was rerun after this adjustment:
338 passed, no failures, one existing ignored test, 67 targets. Final
Clippy and action smoke also pass. Existing tests were not modified or loosened.

## Timings

Each process performs one warmup and five measured calls, sequentially and
pinned to CPU 0 or CPUs 0-3 with one or four Rayon workers. Construction
excludes fixture import, output checks and tessellation. STEP fitting uses
the identical stored `constructed-solid.json` in both versions. Raw samples,
commands, source/binary hashes and machine/compiler details are retained.

| Operation | Parent median, 1 / 4 workers | Updated median, 1 / 4 workers |
| --- | --- | --- |
| Cylinder rim | 2.643 / 2.602 ms, succeeds | 2.468 / 2.470 ms, succeeds |
| Spline-pipe rim | 25.719 / 25.689 ms, succeeds | 25.464 / 26.158 ms, succeeds |
| Spherical meridian | 0.094 / 0.095 ms, fails contact solve | 87.590 / 88.837 ms, succeeds |
| Stored spherical fillet STEP | 132.262 / 137.376 ms, fails fitting | 126.797 / 126.638 ms, succeeds |

Cylinder and spline control mesh volumes are unchanged. Failed operations
are not assigned speedup ratios; the spherical measurements establish the
cost of successful construction/export. These small warmed samples are not
a tail-latency guarantee. The initial closed-wire projection overhead and
its measurements are retained under `initial/`.

Public signatures and diagnostic codes are unchanged. The new solid
coverage establishes the tested constant-radius spherical-meridian cases;
it does not establish arbitrary polar, variable-radius or corner parity.
A clean full native capture follows this implementation commit.
