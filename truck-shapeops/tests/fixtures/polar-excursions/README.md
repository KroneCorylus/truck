# Native spherical cap with a retraced polar seam

`J2.step` preserves raw bytes from
`audits/fillet-chamfer/capture/2026-10-13-conical-domains/artifacts.zip`,
member `blend/simple/J2/capture/001.input.step`.
SHA-256: `247a0f51805008387e6e077454038edb5fbd77c1d94b8f544096531c7d58d839`.

The source creates a radius-50 sphere from latitude -90 to +45 degrees,
closed by the upper plane `h=50/sqrt(2)`, then rounds the upper rim with
radius 10. The spherical face has a cap ring and one meridian traversed
twice in opposite directions, ending at an otherwise isolated south pole.
That retraced excursion contributes no physical trim boundary.

The independent input volume is
`pi*(R^2*h-h^3/3+2*R^3/3)` = **493199.54082823097**.
For the fillet, set `r=10`, `z=h-r`, and
`c=sqrt((R-r)^2-z^2)`. The rolling-circle center is
`(radius,height)=(30.937142421150465,25.35533905932737)`.
Its sphere contact has height `s=R*z/(R-r)=31.694173824159215`.

Below s, the remaining solid follows the original sphere. Between s and h,
its radius is `c+sqrt(r^2-(height-z)^2)`. The primitive of its squared radius,
with `w=height-z`, is

```
F(w) = (c^2+r^2)*w - w^3/3
     + c*(w*sqrt(r^2-w^2) + r^2*asin(w/r)).
```

The remaining volume is
`pi*(R^2*s-s^3/3+2*R^3/3+F(r)-F(s-z))`
= **492592.0503127781**, independent of either kernel's generated result.
The regressions also check the radius-10 rolling circle, normal directions,
both contact limits, unchanged input, topology, closed mesh and STEP at small
and original sizes, including transformed and reversed selections. Their
construction tolerance is 1e-7; official replay tolerance remains 0.001.
