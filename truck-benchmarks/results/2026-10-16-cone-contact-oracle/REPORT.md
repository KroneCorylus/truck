# Independent contact checks explain nine cone discrepancies

The plane/cone replay improvement constructs nine unequal-distance corpus records
whose sampled boundaries disagree with OCCT: chamfer/dist_dist/C1,C6,C7,D1,D2,
D5,D6,D9,E1. Independent contact measurements show that the native output changes
the first requested setback; Truck preserves both requested setbacks. All twenty
raw native-comparison discrepancies remain recorded; no sampled verdict is upgraded.
No geometry, capture, replay or comparison implementation changed in this group.

The oracle matches the native input seed by physical endpoints and a middle point,
requires unique adjacent plane/cone supports and a plane perpendicular to the cone
axis, and derives the original rim from the cone apex, opening axis and half-angle.
For axis coordinate h, radius rho, rim coordinate h0 and radius R:

    planar setback = abs(R - rho), at h = h0
    cone setback = abs(h - h0) / cos(theta), at rho = h*tan(theta).

It inspects 17 points per output edge, accepts only constant-height/radius contact
rings on the original support, and requires exactly one distinct setback on each
support. Native contacts are analytic circular edges, explicitly checked. Truck
STEP contact measurements include the observed within-edge spread in the reported
error. The finite Truck checks are not a certified global error bound.
All 108 source artifacts for the nine corpus cases were byte-verified against the
committed clean full-capture archive before inclusion here.

Corpus results:
- C1 asks for cone0.1/plane0.05. Native cone contact is0.0546918160678.
- C6,D1,D5,D9 ask for cone0.2/plane0.1. Native cone contact is0.1093836321356.
- C7,D2,D6,E1 ask for plane0.2/cone0.1. Native planar contact is0.1093836321356.
- In each case the second requested setback is preserved. Truck preserves both;
  maximum sampled setback error across all25Truck outputs is8.9715758017083758e-09.

Sixteen fresh dimension-defined frustum controls cover both slopes±1, both
reference sides, unequal-distance orders0.3/0.7 and0.7/0.3, and angle30 requests.
All16Truck outputs are valid single-solid STEP, preserve input and match analytic
material. Fourteen native controls match both setbacks and material. Two native
acute-rim controls with first distance0.7 and second0.3 replace0.7 with
0.3043701052459. Their volumes exceed the requested analytic result by1.2113884220
and1.2295199418 despite native validity and stored tolerances around1e-7.
Maximum Truck control volume error is3.0654149441033951e-08. The independent frustum
volume derivation is the same as the plane/cone implementation proof: integrate
the difference of squared radii with radial width a*(1-t/b), b=q/sqrt(1+k*k).

For these45-degree acute rims, the observed first setback equals

    A_effective = A*B / (2*A*cos(beta) - B), beta=pi/4,

when B < A*cos(beta), while the second setback remains B. Every one of the nine
corpus mismatches and two control mismatches agrees with this expression within1e-10.
This behavior is consistent with the native ChFiKPart plane/cone two-distance
conversion taking abs(Dis2-Dis1*sin(angcon)) before atan, losing the angle quadrant.
The relevant native sources and hashes are archived; this is source-supported
attribution, not a modified native rebuild. No native source was changed.

Validation:50measurements (9corpus pairs plus16control pairs), all valid outputs,
exact native contact-ring types, matching analytic controls, and retained mismatches.
The existing39replay tests,13capture tests,Clippy/smoke results remain in the prior
implementation proof at54b272d6. No production change warrants another full capture.
Baseline stays279constructions/259limitedpasses,232failed,32unsupported,96import
failures,9inconsistent;20raw discrepancies, nine now explained by native setbacks.
Archive:424 byte-verified members, SHA256 58bfdd0830bb03f88c010125ab99a5bacddd191c57f2ca06ecb220f28f533e0c.
No full-parity, performance, consumer/pin, push or PR claim/change.
