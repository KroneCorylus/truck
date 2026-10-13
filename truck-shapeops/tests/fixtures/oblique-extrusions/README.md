# Native oblique extrusions

The STEP files preserve raw native input bytes from
`audits/fillet-chamfer/capture/2026-10-13-conical-domains/artifacts.zip`,
members `blend/simple/J6/capture/001.input.step` and
`blend/simple/J8/capture/001.input.step`.

- J6 SHA-256: `cb784f1d18e2005110c36b8dd25195c7bd1bd4872123ce8af5a3f3861ac91698`.
- J8 SHA-256: `03126b75fd1eb4aa31d99f6ab5a039bfef4d472f84ed0e8a8e69efc988cecaa3`.

J6 extrudes the XY ellipse with radii 100 and 50 along `(20,0,100)`.
Its selected upper rim at z=100 receives a radius-10 fillet.
J8 extrudes a radius-50 YZ circle along `(100,100,0)` and rounds the lower
x=0 rim with radius 10. The rigid coordinate map
`(x,y,z) -> (100-y,z,100-x)` puts J8 into the same upper-rim model, with
ellipse radii 50 and 50 and extrusion `(100,0,100)`.
Independent input volumes are `500000*pi` and `250000*pi`.

## Independent fillet volume

The accompanying Python calculation uses only these dimensions, with NumPy
for numerical integration. It does not inspect either kernel's output.
Let `e(phi)=(a*cos(phi),b*sin(phi),0)`, `L=(shift,0,h)`, `k=(0,0,1)` and
`n=normalize(e' cross L)` be the lateral outward normal. For radius `r`,
the side contact parameter is `v=(h-r+r*n.z)/h`, and the rolling center is
`C=e+v*L-r*n`. Therefore `C.z=h-r`.

Set `m=(n-n.z*k)/sqrt(1-n.z^2)` and `alpha=acos(n.z)`. The exact blend is
`F(phi,theta)=C+r*(cos(theta)*k+sin(theta)*m)`, from theta=0 on the cap to
theta=alpha on the side. Its outward area vector is `F_theta cross F_phi`.
The divergence theorem gives the removed volume as

```
[h*(pi*a*b - signed_XY_area(C))
 + integral_phi((1-v) * e dot (e' cross L))
 - integral_phi integral_0^alpha(F dot (F_theta cross F_phi))] / 3.
```

The terms are respectively the removed cap, removed side and replacement
blend fluxes. Analytic derivatives feed periodic trapezoidal integration in
phi and Gauss-Legendre integration in theta. Doubling orders from (256,16)
through (1024,64) changes the results by less than 1e-9:

- J6 remaining volume: **1560464.3531795258**.
- J8 remaining volume: **772406.7218958114**.

The perpendicular circular special case also agrees with the independent
square-minus-quarter-circle formula: removed volume
`2*pi*(R-r*(10-3*pi)/(12-3*pi))*r^2*(1-pi/4)`.

Regression radius and normal checks independently recover phi from a blend
normal `N` as `atan2(b*N.y,a*N.x)` away from the planar contact, then check
the distance and direction from the analytical rolling center. Both cap
and lateral contact normals are checked separately. The tests request
construction tolerance 1e-7; official native replays keep tolerance 0.001.
The native reference volumes differ from these ideal volumes; they are not
substituted as regression expectations, and sampled discrepancies remain open.
