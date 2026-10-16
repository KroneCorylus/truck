# Exact conic contact extensions and their tessellation

A straight-edge chamfer can now extend an exact conic contact edge slightly beyond
its old endpoint, as it could already extend a straight edge. The parameter is
unwrapped beside the moved endpoint; the resulting span must remain positive
and below one complete turn. Both endpoint positions are checked. Extruded
cylindrical supports extend their exact generator using the existing local
conic-extension helper. Face validation and distant-boundary guards remain.
Conic extension is restricted to planar chamfer contact trimming; fillet contacts
and intermediate terminal walks keep their existing behavior.
No API, diagnostic-code, tolerance or dependency change.

Motivating public case: chamfer/dist_angle/F3. Its circular end boundary must
continue from (-50,-86.60254,100) to (-45,-89.3028555,100), inside the rectangular
stem of the original solid. The old trim routine rejected this extension, walked
to the opposite stem boundary and rejected the reversed contact span. The new
intersection and boundary remain exact. General curved selected edges, mixed-edge
junctions and removed support faces remain separate work.

Three tests fail on parent 6ce95353c9a6fc1565e73d37b89a506a928033e4; all four pass unchanged after
the fix. Two geometry tests cover eight equal/unequal, transformed/scaled,
reversed-face-order and reversed-conic-representation variants. Checks include
input immutability, generated face/count, actual contacts, outward bevel plane,
independent material, shared solid checks and robust STEP. Oversized cuts still
reject without mutation. The third parent failure is a frozen geometrically
consistent solid whose STEP matches the independent native control; it isolates
a pre-existing mesh material error independently of the new construction path.

The initial conic implementation produced correct exact geometry but the mesh
volume was 521.9525082408045 instead of 522.9469178406702. The same STEP was native
valid and matched its reference within 4.5e-9 sampled distance. The cylinder mesh
still contained broad triangles after adaptive refinement. The tessellator now
seeds an interior ruling when a grid varies in only one direction and an oblique trim
crosses its parameter grid, so trimming cannot discard all its grid lines.
Isoparametric trims retain their original grid and refinement behavior.
Existing refinement and tolerance checks stay in force. Initial failures and the frozen fixture are retained;
no test assertion was loosened or removed.
The first full run also caught an existing connected-channel fillet regression
when conic extension was attempted on a curved blend. Limiting the new extension
to planar chamfer contacts preserves that existing contract; the unchanged
channel-rim tests cover both fillet and chamfer selections. The failing full-run
log and targeted recheck are retained. The first meshalgo run caught unnecessary
cone-pole vertices from unconditional grid seeding. Restricting the new row to
oblique parameter-space trims preserves the unchanged shared-pole and stepped
extrusion tests. Broad seeding also exposed excessive diagonal error beside an
isoparametric step; its failing log and triangle diagnostic are retained.
Existing isoparametric refinement remains unchanged.

Independent material: radius R = 5, height H = 5, stem half-width a = 2.5 and bottom y = −b = −10.
Let c=sqrt(R²-a²). Initial volume is H*(5*pi*R²/6+2*a*b-a*c).
For top setback d and side setback e, removed material is
 integral(-a..-a+d, e*(1-(x+a)/d)*(b-sqrt(R²-x²)) dx).
The closed-form primitives are
 integral sqrt(R²-x²)dx=(x*sqrt(R²-x²)+R²*asin(x/R))/2,
 integral x*sqrt(R²-x²)dx=-(R²-x²)^(3/2)/3.
Equal 0.25 setbacks give 522.9469178406702; top 0.4/side 0.2 give 522.8987719781295.
Scaling is cubic. This oracle is independent of either kernel's volume output.

Six native controls cover scales 1/0.2, equal and unequal setbacks, both unequal
reference sides. All native and Truck outputs are valid single solids and preserve
input. All match independent material; maximum Truck STEP material error
1.3449152902467176e-10, maximum sampled boundary difference 4.5075708691789085e-13.
The plugin's default native volume integration differs by about 1.03e-6 on the
unit-scale controls. Re-measuring the same immutable native BREP with adaptive
BRepGProp integration at requested 1e-12 accuracy agrees within 3e-11. Both raw and
adaptive values, reported integration-error estimates and the initial assertion
failure are retained. Estimated integration error is not a formal error bound.

Full shapeops release: 400 passed, 0 failed, 1 existing ignored.
Full meshalgo release: 98 passed, 0 failed, 1 ignored.
All 46 replay tests, all-target release Clippy (including meshalgo), single-threaded
actions smoke and final replay build pass. Existing tests/assertions unchanged.
Source/executable hashes are in metadata.json. No performance claim.
Archive: 140 byte-verified members, SHA256 3d8e00ef8586d5a3143ea0bb9523191f4c781a335eba2fc7138b337f7d66b454.
Finite sampling is not a global boundary-error bound or full parity proof.
A clean full capture follows this implementation/proof commit.
No consumer/pin changes, push or PR.
