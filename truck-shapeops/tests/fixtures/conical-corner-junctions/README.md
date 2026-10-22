# Conical corner selections

Native blend/simple/C2, C6, C8 and D1 inputs and physical selections from the
2026-10-22 spherical-corner corpus checkpoint. They cover lower/upper truncated
cone corners, an apex shared by two radial planes, and a lower corner whose
meridian terminates at the remaining radial plane near the apex.

The imported regression preserves captured physical selections, scales geometry
and radii by 0.01, and checks closed material, STEP and input immutability in both
selection orders. Original-scale replay and native comparison remain separately
required. These native outputs have measured comparison discrepancies; they are
not independent material-volume oracles.

Independent material controls integrate the radius-r dilation of an inward cone
intersected with the selected offset planes, clipped to the unselected boundaries.
The 270-degree case adds an independently integrated half-cone with its cap round.
