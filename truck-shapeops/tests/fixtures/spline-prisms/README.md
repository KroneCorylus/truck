# Captured spline prisms

These are unchanged native input STEP bytes for OpenCascade 7.9.3
`tests/blend/simple/G3` and `G7`, captured in
`audits/fillet-chamfer/capture/2026-10-13-cubic-endpoints/artifacts.zip` under
`blend/simple/<case>/capture/001.input.step`.

| File | SHA-256 |
| --- | --- |
| G3.step | f728e433641133019302f26030369b8da0ec77bda7133e422fa3e3e04d66983e |
| G7.step | 84e57136278bfd50f0195b399cb4fc5eba6c1280d2ece7bc0655b96ad768857b |

G3 has a quartic Bezier profile with XY controls `(0,0)`, `(200,0)`,
`(200,50)`, `(250,100)`, `(100,200)`. G7 has a cubic profile with controls
`(0,0)`, `(20,100)`, `(40,60)`, `(100,0)`. Both close with a straight chord
and extrude by `(0,20,100)`.

The independent profile area is half the integral of `x y' - y x'`, plus
the closing chord contribution. Polynomial integration gives signed areas
**23000** and **-3720**, respectively. Thus the positive solid volumes are
**2300000** and **372000**. The oblique extrusion's Y component does not
change the volume; its component normal to the profile plane is 100.

The STEP support uses a unit extrusion direction. Its trimmed boundaries
extend about 102 parameter units in the opposite direction, beyond the old
stored interval `[0,1]`. The tests require the complete bounds, consistent
geometry, independent material, closed meshes and STEP round trips, including
small/original scales and rigid motion. Separate importer regressions cover
nonplanar polynomial/rational profiles, interior boundary extrema, reversal
and affine frames.
