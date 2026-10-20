# Tilted cap chamfer inputs

Independent OCCT inputs captured on 2026-10-20: a unit block with its lower-right vertical corner rounded at radius 0.4, then cut by `z = 0.782539615254126 - tan(angle) * x`. Angles are 0, +2, -2 and +10 degrees. JSON retains the native three-edge contour and planar reference for cap/wall distances 0.1/0.2. Tests also reverse the contour and use the equivalent cylindrical reference with swapped distances.

Input volume is `h * (1 - R² * (1 - pi/4)) - tan(angle) * (0.5 - R² * (1 - pi/4) * (1 - c))`, where `R=0.4` and `c=R*(10-3*pi)/(12-3*pi)`. For the horizontal control, the chamfer removes `a*b/2 * (1.2 + pi/2 * (R-a/3))`, with cap setback `a` and wall setback `b`.

The replay's coarse mesh volume is preserved against its parent values, while refined tessellation checks the independent horizontal volume. Tilted material uses the declared native comparison tolerance. Exported contact checks distinguish the selected supports from terminal trim faces.

Generation source, raw native references, parent failures, source hashes and retained native distance discrepancies are archived in `truck-benchmarks/results/2026-10-20-tilted-cap-unequal/`. The +10-degree references and one +2-degree reference fail the unchanged sampled distance comparison; these fixtures are not a claim of native parity.
