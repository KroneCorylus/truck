# Cone reference faces and circular plane/cone chamfers

Face-dependent chamfers on plane/cone circular rims were rejected: capture omitted
conical reference identity and replay supported only planar or plane/cylinder
support pairs. Schema 2 now optionally records reference_cone with apex, opening
axis and positive half_angle_radians, retaining reference_plane:null. Replay
matches exactly one adjacent seed face by apex, opening direction and cone angle.
The nappe matters: reversing a cone axis or shifting its apex is not equivalent.
Native negative semiangles are normalized by reversing the captured opening axis.
Existing contour-side propagation preserves distance ordering.

Plane/cone support conversion is restricted to planes perpendicular to the cone
axis. For oriented plane normal n, cone opening axis u, half-angle theta and cone
orientation sign s, cos(beta)=s*n.dot(u)*sin(theta), sin(beta)=cos(theta).
The existing sine rule converts d1=d0*sin(alpha)/sin(beta+alpha).
Unsupported pairs, impossible angles and malformed/ambiguous references remain
explicit failures. No geometry solver, public API, fit tolerance or scope changed.

Three new regressions fail on parent 97b900787a4d4d685fc2edc959ab6d2467e2d577 and pass unchanged.
Sixteen constructed variants cover both cone slopes, both reference sides,
unequal and distance-angle requests, scale, rigid transforms and reversed samples.
They check exact side assignment, immutable input, analytic material, closed
geometry/mesh and robust STEP through shared harnesses. Guard cases cover wrong
apex, opposite nappe, wrong angle, invalid/overflowing axes and ambiguous metadata.
No existing assertion or test was changed, ignored or loosened.

Eight fresh native OCCT 7.9.3 controls use full circular frusta of height 10,
bottom/top radii 5/3 and 3/5, planar/conical reference, unequal distances 0.3/0.7
and distance 0.3 with angle 30 degrees. Native validity, one contour, input BREP
immutability and independent material pass. The same descriptor harness fails
against the parent plugin on missing reference_cone, then passes with the new
plugin. Other task fields and native result properties remain identical.
Fixture bytes originate in the earlier exploratory controls, retained separately
as fixture-origin; synthesized replay variants are explicitly identified.

For top radius R, signed slope k=dr/dz, planar inset a and cone setback q, let
b=q/sqrt(1+k*k). At depth t the uncut radius is R-k*t and the chamfer removes
radial width a*(1-t/b). Integrating the difference of squared radii gives:

    removed = pi * (R*a*b - k*a*b*b/3 - a*a*b/3)
    input = pi*H/3 * (Rbottom² + Rbottom*Rtop + Rtop²).

These formulas depend only on dimensions and requested parameters. All eight
Truck replays export native-valid single-solid STEP, preserve input and agree
with independent material. Maximum sampled boundary difference: 1.4734521025516756e-07;
maximum analytic STEP volume difference: 4.548024890027591e-08. Sampling is not a global bound.

Validation: all 39 replay tests, 13 capture tests, all-target release Clippy,
single-threaded action smoke and final replay build pass. The unchanged geometry
solver's earlier 391-pass full suite remains recorded at shared-chamfer-ends;
this capture/replay change did not rerun that full suite. Hashes are in metadata.json.
Archive: 378 byte-verified members, SHA256 1ee0a366d8f7f12534bd60bf2b616c4f9575c7c91b6a72062f96ef9057127481.
A clean full corpus capture follows this implementation/proof commit.
No full-parity, speed or corpus-recovery claim is made by these controls.
No consumer/pin changes, push or PR.
