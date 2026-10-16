# Plane-reference angles on longitudinal cylinder edges

The replay adapter now derives a cylindrical contact from the requested planar
setback and angle for a straight edge parallel to a cylinder axis. It starts at
the planar contact and intersects the inward angle ray with the cylinder. The
nearest positive intersection must remain on the existing local tangent branch
and inside the cylindrical face direction. Its chord then uses the existing
chord-to-tangent conversion. The kernel itself is unchanged. Cylinder-reference
angles remain explicitly unsupported; their native conventions need separate
investigation. Existing perpendicular rims and planar sine-rule conversion stay
unchanged. No API, diagnostic-code, tolerance or dependency change.

Three tests fail on parent f3c67ee9a905873fb3d97e90afb738f895c1e629; all four tests pass after the
change. Twenty constructed variants cover convex and concave cylinders, planar
offsets -2/0/+2 at30 degrees, central planes at15/60 degrees, scale0.2, rigid
transform, reversed face order and reversed captured point order. Assertions
check assignment sides, actual planar/cylindrical contact vertices, outward bevel
plane, input immutability, independent material, shared solid checks and STEP.
A fourth test rejects cylinder-reference requests, nonlocal contacts and rays
that miss the cylinder without input mutation.

The initial test oracle used the convex outward normal for concave fixtures.
Its volume and contact assertions passed, but its bevel-normal assertion failed.
That test-only sign was corrected, all final tests were rerun on the parent
before restoring implementation, and the final regression text hash is identical
between parent and fixed runs. Initial parent/fixed logs are retained. No existing
test or assertion was removed, ignored or weakened.

Independent section oracle (R5,H10): the plane is y=h and the seed is
S=(sqrt(R²-h²),h). The planar contact is P=S+(s*0.7,0), with s=-1 convex and
s=+1 concave. Intersect P+t*(-s*cos(alpha),sin(alpha)) with the radius-R circle
and take its first positive local contact Q. Let psi be the angular change S→Q.
The removed section is abs(cross(S,P)+cross(P,Q)-R²*psi)/2. Multiply by H and
subtract from the original circular segment, or its rectangular complement for
the concave solid. This material calculation is independent of kernel volumes.

Ten native controls cover six offset/convexity cases at30 degrees and four
central15/60-degree cases. All native and Truck outputs are valid single solids;
all inputs remain byte-identical. All Truck outputs preserve requested angle,
contacts and material, with maximum STEP material error 8.8448359747417271e-11, maximum
contact error 1.5987211554602254e-13, maximum angle error 4.5474735088646412e-13 degrees.
For eight controls the native output also satisfies the request; their maximum
sampled Truck/native boundary difference is 2.007283228522283e-13.

Two native controls violate the requested plane angle despite valid topology:
- convex plane h=+2: requested30°, analytic native bevel47.294898328645274°;
  native volume error -0.6298351858989406.
- concave plane h=+2: requested30°, analytic native bevel20.511460028020405°;
  native volume error +0.8598790045034548.
Both keep planar setback0.7. The analytic plane extractor and contact vertices
agree; this is not a tessellation discrepancy. Raw sampled differences
0.18824717504059094 and0.15012201333725517 remain recorded, not upgraded to passes.
The native plane/cylinder routine computes an unsigned square root for an angle
component; that is a source-level lead, not a completed source attribution or fix.
The archive retains the original12-control investigation, including unresolved
cylinder-reference controls. The validated claims above concern only ten
plane-reference controls. No native source was patched for this achievement.

All46 replay tests,13 capture tests, release all-target Clippy, single-threaded
actions smoke and final replay build pass. Core source and integration-test bytes
match ac5b8fa3's full396-pass/0-fail/1-existing-ignored shapeops validation, reused
without a redundant run. Source/executable hashes are in metadata.json.
Archive: 367 byte-verified members, SHA256 d517a1af10ef7026c9c825f46860895001671f9d4e0c32ef745f11ebd45c1ecc.
Finite sampling does not prove full parity or a global distance bound.
A clean full capture follows this implementation/proof commit.
No consumer/pin changes, push, PR or performance claim.
