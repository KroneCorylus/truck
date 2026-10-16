# Planar chamfers crossing curved terminal boundaries

The straight-edge blend solver already walks past consumed short neighboring
edges, but rejected every curved edge encountered between its terminal faces.
A planar chamfer now uses the existing curve/surface intersection routine for
that boundary. The result must stay within the original curve's parameter range;
existing trimming, topology and support-face validation remain in force. The
straight-boundary path is unchanged. Curved terminal walks for nonplanar blend
surfaces remain unsupported. No new solver, tolerance, API or diagnostic code.

Motivating corpus case: dist_dist_sequence/A6/005, newly reached after native
chord conversion. Its nine-face input has a cylindrical support, planar support
and conical end faces. The terminal walk crosses a quadratic B-spline on a cone.
The direct route rejected that curved boundary at planar_edge.rs line226; later
wire fallback reported trim_faces instead. The stable baseline capture at
b5c98d1a records this failure and complete unchanged physical selection.

Two positive regressions fail on parent 4d8548cc237360e2ae855170904eaf647a24fff8 and pass unchanged
after the fix. Four constructed variants cover equal/unequal kernel distances,
rigid transform, scale0.2 and reversed face order. A quarter-annular cylinder
has radius1, bore radius0.25, length1 and a conical end of length0.2 tapering to
radius0.95. The new bevel consumes the short radial edge and crosses the circular
rim onto the end cap. Checks include unchanged input, retained face count plus
one bevel, consumed edge removal, actual rim intersection, bevel plane, independent
material, shared solid and robust STEP. A third test verifies six oversized
requests reject without input mutation; it passes both parent and fixed versions.
Existing general chamfer, consumed-neighbor and plane/cylinder tests pass unchanged.

Independent material oracle:
For plane intercept a, chamfer angle alpha, and section radius r, the ray from
P=(a,0) along (cos(alpha),sin(alpha)) meets the circle at Q=(y,z), with
 t=-a*cos(alpha)+sqrt(r²-a²*sin(alpha)²), y=a+t*cos(alpha), z=t*sin(alpha).
The removed section is A(r)=(r²*atan2(z,y)-a*z)/2. The exact body volume is
 pi/4*(1+0.2*(1+0.95+0.95²)/3-1.2*0.25²).
Remaining material is that volume minus A(1) and
 integral(1..1.2, A(1-(x-1)/4) dx).
The archived oracle evaluates this analytic integral at40 and80 decimal digits
using two quadratures; the results agree at their recorded precision. This is
independent of either kernel's computed volumes, not a formal quadrature bound.
Equal kernel distances0.2 give volume0.8524577031471755874968;
plane0.1/tangent0.203058660634 give volume0.8639619850312714808347.
The uncut volume is0.8758498518820544399397. Scaling is cubic.
mpmath is an isolated proof-tool dependency, not a production or CI dependency.

Six native controls cover scales1 and0.2 and three requests: native equal chord
setbacks0.2; plane0.1/cylinder chord0.2; and plane0.2/cylinder chord
2*sin(atan(0.2)/2), which corresponds to equal kernel tangent offsets0.2.
All native and Truck results match independent material and are valid single
solids. Truck replays preserve input and export valid single-solid STEP.
Maximum sampled boundary difference: 4.049468120209827e-08; maximum analytic STEP volume
error: 7.3791972532433192e-11. Finite sampling is not a global boundary-error bound.

Full shapeops release: 396 passed,0 failed,1 existing ignored.
All42 replay tests, all-target release Clippy, single-threaded actions smoke and
final replay build pass. Existing tests and assertions are unchanged.
Source/executable hashes are in metadata.json. No performance claim.
Archive: 123 byte-verified members, SHA256 4eadd2b9b37cb98b95be422cded6a6ed47b0b2f16028df73c31e1a6de27f84b5.
A clean full capture follows this implementation/proof commit.
No consumer/pin changes, push or PR. General complex junctions, removed support
faces, nonplanar curved-boundary walks and distance-angle semantics remain work.
