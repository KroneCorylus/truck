# Remaining package C junction triage

Read-only evidence review, 2026-10-24. No builds, native execution or checkout changes. Exact rows are blend/simple/{F9,G4,G8,I2}/001. rows.json preserves selection endpoints, source programs/hashes, original task hashes and archived failures. Evidence is the frozen 2026-10-24 mixed capture, not the currently running capture.

## Common diagnostic issue

All four archived failures report along-wire tangent discontinuity. replay_occt_blend.rs around1280 overwrites the multi-edge error with try_fillet_solid_along_wire when the selected edges form a wire. These intentionally distinct contours are not tangent chains. The next diagnostic must call try_fillet_solid_edges_with_radii directly and record that error before any wire fallback. Do not relax tangency, change selected edges or infer the inner guard from the final replay message.

## F9 / G4 / G8: two straight selected edges, common planar wall

These share topology: selected longitudinal ruling and selected upper closing chord meet at a corner; the curved upper cap/profile edge stays unselected. Opposite supports are the extrusion and upper cap. This differs from G2's selected curved cap edge plus straight cap edge, whose common support is the cap. G2's scoped helper cannot be reused by merely changing an index: its geometry requires the selected cap/extrusion curved edge, whereas these select a straight extrusion/wall ruling.

F9 is the smallest next case. Original units: profile B(s)=(0,100s,50s²), -1<=s<=1, closed by z=50; extrusion A=(20,0,100); radius1. Selected edges meet at (20,-100,150), extending to (0,-100,50) and (20,100,150). Captured scale10 and radius10. This is a quadratic symmetric extrusion and orthogonal pair of selected straight edges, with exact source volume400000/3. Existing F5 recognition alone does not assemble a two-selected junction, and G2 eligibility does not cover this orientation.

G4: degree4 Bezier controls (0,0,0),(20,0,0),(20,5,0),(25,10,0),(10,20,0), sweep(0,2,10), original radius1. Common endpoint is the first basis endpoint at the swept cap. Pair angle100.10261179deg; asymmetric higher-order curvature means the F9 reflection/monotonic assumptions cannot be carried over.

G8: clamped cubic with controls (0,0,0),(2,10,0),(4,6,0),(10,0,0), sweep(0,2,10), original radius.5. Selected pair orthogonal but basis asymmetric. Single-span polynomial representation is simple; geometry/branch proof is still distinct from F9 symmetry.

## I2: different elementary surface network

Native constructs a quarter circular pipe: triangular profile (0,0,0),(10,0,0),(5,0,5) around circle centered(-20,0,0), radius20, angle0..pi/2. Equivalent exact body is a quarter revolution of triangle rho=20..30,z=0 with apex rho25,z5. It has two conical sides, base plane and two radial end planes, not an extrusion. Source volume (pi/2)*25*25 =981.74770424681 original units³ agrees with archived source after scale10.

Selected edges are inner base circular rim rho20,z0; start radial base segment rho20..30,z0; and start outer slant rho30,z0 to rho25,z5. Thus selected pairs meet at two different vertices: inner-cone/base/start-cap and outer-cone/base/start-cap, with a remote one-edge termination at the quarter-turn end and at the top ridge. Existing planar_fillet cone branch explicitly accepts chosen.len()==1 or3, so both two-selected corners are outside that local branch. The top ridge also joins both cones and an end plane; its exact dispatch/import recognition needs direct tracing. Do not label this simply the same extrusion mechanism or claim the cone guard is the final public API error without a probe.

## Smallest reproducible next work

1. F9 procedural source using exact quadratic and sweep above; choose the two physical straight-edge midpoints (10,-100,100) and (20,0,150). First preserve direct multi-edge error with immutable input, then intended positive regression at radii .5,1,1.5, both selection orders and rigid/uniform-scale placement. Parent failure must be construction rejection after valid-source/selection checks, not the replay fallback.
2. Independent F9 oracle must be derived before a positive volume literal is accepted. Start exact source inequalities 0<=x<=20, 5x+y²/200<=z<=5x+50. Derive retained surface trims and junction patches from the two selected support-pair offsets (extrusion/wall and cap/wall), respecting the unselected curved cap edge. Integrate divergence flux using independent polynomial/analytic offset equations, verify oriented area closure and convergence across quadrature orders, zero-radius source volume and monotone removal. Do not assume a three-edge sphere or copy the G2 four-patch boundary; the cap/extrusion edge is sharp here. Native volume is comparison only.
3. Separately create exact I2 quarter-revolution input and first verify source volume625*pi/2 (=981.74770424681). Map the three selected witnesses directly, then probe each two-edge pair and the full set. Independent material can use cylindrical volume element rho d rho dtheta dz with exact cone/plane offsets and explicit end-junction trims. No positive-radius material value is established yet; first identify which local piece is absent.
4. G4/G8 follow only once F9 has a valid construction/oracle. Their source-area integrals are exact polynomial Green integrals, useful source controls, but not positive-radius proof. No general junction abstraction is justified yet.

Existing evidence: checkpoints/2026-10-24-mixed-curved-corners/cef-triage-2026-10-24.txt already identified the three extrusion rows and masked diagnostic. This review confirms exact tasks and adds the I2 conical network distinction. No new success or parent-failing execution claim.
