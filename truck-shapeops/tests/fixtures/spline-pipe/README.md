# Closed spline pipe input

Generated from OCCT V7_9_3 `tests/blend/simple/J9` by the fillet/chamfer capture
harness before its blend operation. The source sweeps a radius-50 disk along
a cubic Bezier with control points (0,0,0), (100,0,0), (100,100,0), (200,100,0).
OCCT exports a padded periodic rational spline representation for the circular
direction. Captured input provenance is retained in
`audits/fillet-chamfer/capture/2026-10-12-unclamped-domains/artifacts.zip`.

The independent ideal sweep volume is pi * 50² * the cubic's arc length:
1815077.75712917, using converged Simpson quadrature. The captured approximation
has native volume 1815077.51635755; the difference is below the regression's
physical tessellation allowance. The regression checks the imported solid
and STEP round trip at scales 0.01 and 0.025, including a rigid transform.
`spline_pipe_blend.rs` also checks the start-cap rim at fillet radii 10 and 5,
including a scaled, rigidly transformed and reversed wire. It checks radius and
normal alignment on the generated patches, geometry consistency, closed topology
and tessellation, independent volume, STEP round trip and immutable input.

## Independent rounded volume

`independent_blend_volume.py` uses only Python's standard library. It evaluates
the ideal tube and rolling-sphere envelope directly, without Truck or OCCT output.
The captured spline approximates this ideal surface, so the expected volume is
compared using the existing physical tessellation allowance.

Let C(t) be the cubic centerline, T its unit tangent, N=(-Ty, Tx, 0), B=(0,0,1),
s its speed and kappa its signed curvature. For tube radius R=50 and fillet radius
r, define rho=R-r and n=N cos(theta)+B sin(theta). The sphere-center locus is
K=C(tc)+rho n, where tc solves Kx=r. Implicit differentiation gives

```
tc' = -rho Ty sin(theta) / [s (1-rho kappa cos(theta)) Tx]
K' = s (1-rho kappa cos(theta)) T tc' + rho (-N sin(theta)+B cos(theta))
```

Set U=(0,ny,nz)/hypot(ny,nz), V=(0,-Uz,Uy), k=K' dot V,
omega=U' dot V and beta=acos(-nx). The blend envelope is
P=K+r[-X cos(alpha)+U sin(alpha)] for alpha in [0,beta].
The divergence theorem with flux F=(x,0,0) gives the removed volume by subtracting
the blend flux from the replaced tube-wall strip. The cap at x=0 contributes no
flux; every other unchanged surface cancels.

```
old strip = integral(t=0..tc) x (-Ty cos(theta)) R s (1-R kappa cos(theta)) dt
blend flux = -r² [k I0 + r omega I1]
I0 = sin(beta) - beta/2 - sin(2 beta)/4
I1 = 1/6 - cos(beta)²/2 + cos(beta)³/3
removed = integral(theta=0..2pi) (old strip - blend flux) dtheta
```

The script uses Simpson quadrature and 60 bisections for each contact station.
With angular divisions 128, 256 and 512 (one quarter as many strip divisions):

| Radius | 128 | 256 | 512 |
| --- | --- | --- | --- |
| 5 | 1813443.8360017312 | 1813443.8360016774 | 1813443.8360016735 |
| 10 | 1808730.962401204 | 1808730.9623990662 | 1808730.9623989318 |

The regression uses the final column. This establishes these two constant-radius
cases; it does not establish general freeform or variable-radius parity.
