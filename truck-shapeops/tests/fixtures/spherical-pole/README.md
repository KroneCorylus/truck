# Spherical pole input

The captured input of OCCT V7_9_3 `tests/blend/simple/E3`: a radius-150
sphere quadrant, from latitude -90 degrees to 60 degrees, rotated 90 degrees
around Z. It occupies x <= 0, y >= 0 and z <= 150 sqrt(3)/2. The selected
meridian lies on y=0, from the south pole to the upper cap. The requested
fillet radius is 10. Input provenance is retained in
`audits/fillet-chamfer/capture/2026-10-12-spline-rim/artifacts.zip`.

`spherical_pole_blend.rs` checks scales 0.02, 0.04 and 1, with rotation,
translation and reversed selection in the second case. It requires analytic
rolling radius and normals, geometric consistency, closed topology/mesh,
independent volume, STEP and unchanged input. The two smaller cases fail on f7f8f4f5; the original scale also exposes
a singular projected-curve seed in the input mesh.
A separate exact contact-circle test checks the center and both contact
points at the pole with both curve/support orders and both radius signs, within 1e-10.

## Independent volume

For sphere radius R and fillet r, put h=R sqrt(3)/2 and c=sqrt(R²-2Rr).
The original quadrant volume is pi/4 (R²h-h³/3+2R³/3). In each plane of
constant y, the retained section is the portion of a half disk below z=h.
For disk radius q greater than h, its area is

```
A(q) = [h sqrt(q²-h²) + q² asin(h/q) + pi q²/2] / 2.
```

The fillet section has y=r(1-cos(alpha)), radial extent c+r sin(alpha)
and alpha from 0 to acos(-r/(R-r)). Subtract its area from that of the
original sphere section, whose radius is sqrt(R²-y²), and integrate with
dy=r sin(alpha) d(alpha). This is a direct geometric volume calculation,
independent of either kernel's output. `independent_volume.py` records
Simpson convergence; the Rust regression uses 4096 intervals. The selected
radii keep every relevant disk radius greater than h.

`truck-stepio/tests/fixtures/spherical-pole-fillet.json` stores the procedural
fillet face generated from this input with the polar contact initialization
fix, before changing the export fitter. Its dedicated regression fails the
parent surface fitter and checks a dense independent parameter grid at
0.00005/sqrt(3), the unchanged local export tolerance. It also checks the
193-control budget and legal cubic knot multiplicities.
