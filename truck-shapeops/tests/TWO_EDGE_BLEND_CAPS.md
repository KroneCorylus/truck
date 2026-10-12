# Two-edge curved blend caps

`two_edge_blend_caps.rs` covers open fillet wires where a supporting face is
bounded by the selected curved edge and one other edge. Both contacts trim that
same other edge, and both fillet ends trim the same terminal face. The retained
middle edge and both terminal intersections must be assembled together.

The half-cylinder has radius R, height H, and a fillet of radius r on its upper
semicircular rim. Integrating the removed meridian area with the cylindrical
Jacobian over a half revolution gives

```
removed = pi/2 * (2 R r² - 5 r³/3 - pi/2 (R-r) r²)
volume = pi R² H/2 - removed.
```

The parabolic prism is the normalized geometry of OCCT 7.9.3
`tests/blend/simple/G1`. The direct regression uses an exact quadratic Bezier
edge and a translation sweep, independent of STEP import. Its material is

```
0 <= x <= 0.2, -1 <= y <= 1, y²/2 + 5x <= z <= 0.5 + 5x.
```

The input volume is 2/15. A radius-1 rolling ball rounds the parabolic edge on
x=0.2. Write q for the y coordinate of contact on the parabolic support,
L=sqrt(26+q²), M=sqrt(1+q²). The ball center and fillet section are

```
C = (-0.8, q(1-1/L), q²/2 - 4 + 26/L)
P(q,g) = C + (cos(g), q/M sin(g), -sin(g)/M)
0 <= g <= atan(M/5).
```

The radial vector is the expected unit surface normal. At g=0 the ball touches
the cap; at the other limit it touches the parabolic support. The regression
inverts P.y by bisection, then independently checks the radius and normal.

For volume, project the removed material onto the xy plane. Its thickness is
`min(P.z - P.y²/2 - 5 P.x, 0.5 - P.y²/2)`, clipped below at zero. The absolute
projection Jacobian is

```
sin(g) * (1 - 1/L + q²/L³ + sin(g)/M³).
```

Integrate their product and double by symmetry. q ranges from zero to the
positive solution of `q(1-1/L)=1` (about 1.2354845617895887). For q<=1 the upper
g limit is atan(M/5); for q>1 it is asin((1-C.y)M/q). This includes sections
beyond the contact endpoints, where the top plane clips the curved patch.
Split the inner quadrature at its intersections with that plane. Split the
outer quadrature where it first touches the cap, at q=1, and where the plane
is tangent to a section. The recorded adaptive Simpson calculation uses these
explicit analytic functions, without querying either kernel's result.

At quadrature tolerances 1e-9, 1e-11 and 1e-13, normalized volumes are
0.13218565097594601, 0.13218564998431082 and 0.13218564998388772. The final two
differ by less than 4.3e-13. Tests use the last value, scaled by scale³, and the
common harness's tessellation error allowance. They also check topology,
closed meshes, STEP, unchanged input, rigid transformation and reversed wire.

The original capture uses scale 1000. Its repaired Truck STEP volume is
132185649.97246, versus independent 132185649.98388772. OCCT's captured result
has volume 132189932.40065464 and does not match the repaired surface within
the audit's 0.001 sampled-distance threshold. This discrepancy is retained in
the audit; successful construction is not counted as full case parity.
