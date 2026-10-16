# Close both chamfer ends on a shared terminal face

Open-wire chamfer assembly previously rebuilt each terminal face using only one
end bridge. When both ends reached the same face, all support cuts were already
present but the second gap had no bridge, so the operation failed at trim_end_faces.
Assembly now groups the end junctions by original face, applies both bridges in
one reconstruction, and requires each bridge to close exactly one gap. Contact
fitting, distance ordering, public APIs and diagnostic codes are unchanged.

Both new regressions fail on parent 18232e095c86fe892390c74398136c16d32c0776 at trim_end_faces.
The exact same tests pass after the fix. They construct a half-cylinder whose
upper cap is one semicircular edge plus one diameter. Both chamfer ends trim
that diameter and the same rectangular terminal face. Equal and unequal setbacks
are exercised under rigid transforms, wire reversal with exchanged assignments,
and scales1 and0.1. All six variants check unchanged input, consistent geometry,
expected face count, independent cone equation and outward normals, analytic
material, closed topology/mesh and the shared robust STEP harness.
No existing assertion or test was changed, ignored or loosened.

For radius R,height H,radial inset a and vertical setback b, the input volume is
pi R² H/2. At depth t below the top, the radial inset is a(1-t/b). Integrating
the removed half-annulus gives

    removed = pi/2 * (R a b - a² b/3),
    remaining = pi R² H/2 - removed.

The chamfer lies on rho-R+a+(a/b)(z-H)=0, with outward normal proportional to
(x/rho,y/rho,a/b). These formulas use fixture dimensions, not native output.

Three fresh native OCCT7.9.3 controls use the same four-face input topology.
The generator sweeps an arc/chord face, avoiding the extra split radial faces
of a primitive partial-cylinder construction. Captures preserve shared seed/face
identity through a compound BREP and record one complete chamfer contour.
All three native outputs are valid single solids with independent volume;
all three Truck replays preserve input, construct consistent closed meshes and
export native-valid single-solid STEP. Maximum sampled boundary distance is
6.9669054156017117e-09; maximum Truck STEP analytic volume difference is2.1378127712523565e-08.
These finite checks are not a certified global-distance or full-parity result.

Full shapeops release: 391 passed,0failed,1existingignored.
All33replaytests, all-target release Clippy, single-threaded action smoke and the
final replay build pass. Source/executable hashes are frozen in metadata.json.
The original-scale ellipsoid and polar tests remain unchanged in this full run.
No performance claim is made; smoke execution is not a latency benchmark.

Archive:116 byte-verified members, SHA256 821b94cd0ef4b3a87dff48fa50d4a469466153f122aa8860dbc374e49a143264.
It retains raw final native controls, the initial native run and helper scripts.
The initial native run itself passed; its new validation harness looked for an
obsolete contour key instead of schema2 contour_edges. The corrected harness
checks the captured one-edge contour, validity, analytic material and input bytes.
A clean full corpus capture follows the implementation/proof commit.
No consumer/pin change, push or PR. General multi-face terminations, distant
collisions and consumed supporting faces remain outside this fix.
