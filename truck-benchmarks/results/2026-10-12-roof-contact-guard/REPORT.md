# Planar trim validation after blending

At parent c654bfe4, captured OCCT blend/simple/Q4 operation 2 constructs a
geometrically consistent, topologically closed Truck result with intersecting
planar boundary loops. Its mesh is open; STEP reimport in OCCT yields two solids.
The selected post-root fillet reaches a rounded roof edge beyond its planar
support. The previous construction treats that support as an unbounded plane.

Modeling blend completion now reuses the planar boundary validator for changed
planar faces retaining their loop count. Intersecting contact loops, reversed
regions and invalid hole nesting return OutsideNeighbour at validate_trims.
This closes a false-success path; rolling the ball onto adjacent curved support
is still missing. Curved-face domains and general distant collisions are not
validated by this repair. No API or diagnostic code meaning changes.

The original Q4 STEP input is preserved byte-for-byte as
truck-shapeops/tests/fixtures/rounded-roof-post/input.step. The regression checks
input topology, a closed mesh and independent volume
125 - 5*(1-pi/4) + 7.5*pi. It rejects the invalid radius-1 output before and after
a rigid transform/reversed selection, and verifies a radius-0.2 fillet that fits,
including independent volume and STEP roundtrip. Its guard assertion fails on
the parent; input and smaller-radius checks already pass there.

Validation: full release truck-shapeops suite 328 passed, zero failed and one
existing ignored test across 62 targets. All-target release Clippy and action
benchmark smoke pass. No tests were removed, ignored or loosened. This batch
makes no performance claim.
