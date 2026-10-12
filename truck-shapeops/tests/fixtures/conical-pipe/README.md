# Conical pipe end blend

Captured OCCT V7_9_3 `tests/blend/simple/I3` input, before its radius-10 fillet.
The pipe is a quarter revolution about the z axis through (-200,0,0).
At height z in [0,50], its radial interval is [200+z,300-z]. Its exact volume
is 312500 pi. The selected edge joins (-200,200,0) to (-200,300,0).
The captured source and native output are retained in
`audits/fillet-chamfer/capture/2026-10-12-spline-seams/artifacts.zip`.

The rounded corner is a radius-10 cylinder along y, centered at x=-190,z=10.
Using X=x+200, the removed cross-section has X=10(1-cos t),
Z=10(1-sin t), for t in [0,pi/2]. At (X,z), the material's y thickness is
sqrt((300-z)^2-X^2)-sqrt((200+z)^2-X^2). Integrating that expression over
z in [0,Z], then X, gives removed volume 2050.297332421.
The regression's independently converged Simpson integral gives retained
volume 979697.406914389; the native output volume is 979697.406915526.

The regression checks original size and a 0.01 scale with rigid transformation
and reversed edge selection: consistent geometry, a bounded coarse mesh,
closed fine meshes, topology, independent volume, STEP, and immutable input.
