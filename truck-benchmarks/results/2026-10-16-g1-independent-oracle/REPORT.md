# G1: independent rolling-radius and contact checks

The captured native G1 fillet deviates from its independently derived constant-
radius envelope at sampled points. The Truck STEP is much closer to that envelope.
This explains why reproducing the captured native shape is not a suitable sole
correctness target for this case. No kernel, replay, capture or comparison code,
threshold, status or parity verdict changed. G1 remains one of eleven sampled
shape discrepancies; the corpus remains 231 constructions / 220 limited passes.

## Verified geometry and oracle

G1's four native input faces are the planes x=0 and x=200, the plane z=5x+500,
and the extruded parabola z=y²/2000+5x. The diagnostic verifies the native curve
is a parabola of focal length500 at the origin, with its axis along +z and its
transverse axis along either sign of y; the extrusion is parallel to (1,0,5).
It checks the four face equations on a 33×33 grid (maximum residual2.28e-13),
face coverage and independent input volume400000000/3. Captured final contour
radius is1000. Each output has five faces, exactly one B-spline fillet face;
only that face and its boundary edges enter the radius/contact diagnostics.

Let q be y at contact with the parabolic support and g=sqrt(26+q²/1e6).
The outward support normal is (5,q/1000,-1)/g. Offsetting the plane x=200 and
the parabolic support inward by1000 gives the center locus

    C(q)=(-800, q(1-1/g), q²/2000-4000+26000/g).

The constant-radius envelope satisfies distance(P,C)=1000. Its plane contact
is (200,C.y,C.z). The probe finds the nearest center by bisection of

    h(q)=C.y-P.y + q/1000*(C.z-P.z).

C'(q)=(1-26/g³)*(0,1,q/1000). For sampled P.z<=1500.000001,
h'(q) >= (1-1/sqrt(26))+(C.z(0)-P.z)/1000 >0.4029.
Thus the root is unique; the distance derivative has its sign, so the root is
the global closest center. The probe rejects points outside this proof domain
and roots not bracketed by [-2000,2000]. This is an analytic minimization argument,
not an assertion that finite surface samples certify an entire shape.

Controls sample4635 exact envelope points and points shifted outward by0.01.
Exact radial residual and the error in measuring that0.01 shift are below3.6e-13;
exact plane-contact residual is below4.6e-13. Invalid-domain/bracket controls fail
as expected. Input BREP serialization is identical before and after both native
rebuilds, which operate on independent copies.

## Measurements

Three grids (32,64,128 intervals, with twice as many boundary stations) agree
on the reported worst radial discrepancy. The densest run samples17,219 native
and17,205 Truck fillet/boundary points, including380 plane-contact points each.

| Sampled diagnostic | Captured native | Truck STEP |
| --- | ---: | ---: |
| Maximum absolute radius residual |0.1430554788634|0.0000045934381|
| Maximum nearest plane-contact curve distance |0.3487193942755|0.0002149467586|
| Radius residual with nearest-center abs(q)<=1000 |0.0221812395291|0.0000029096207|
| Radius residual beyond that interval |0.1430554788634|0.0000045934381|

The largest native radial residual is near(186.0959228,969.4527378,1430.4796140),
with nearest q=1050.2135406, in the end extension. Smaller discrepancies also
occur inside abs(q)<=1000, so the issue is not restricted to extrapolated ends.
The additional plane_contact_residual field is a same-y z difference; it is
NOT Euclidean distance. The table uses plane_contact_distance instead.

Native rebuilding with the captured settings reproduces these values. Tightening
all SetParams inputs to1e-8, except angular1e-6, and continuity angle1e-6 still
produces a native-valid solid with essentially the same discrepancies (change
below1e-6). This experiment does not identify the internal native algorithm cause.
Stored native tolerances remain separate from these actual analytic residuals.

The earlier parent-failing regressions committed in83eccc89 already check this
parabolic construction and a half-cylinder through shared solid/STEP harnesses,
including independent normals/material, transforms, reversal and unchanged input.
Both existing tests pass again unchanged. Their analytic material derivation is
in truck-shapeops/tests/TWO_EDGE_BLEND_CAPS.md. The current captured Truck STEP
volume132185649.9747912 differs from the independent converged value
132185649.98388772 by about0.00910; this is supporting material evidence, not a
certified global-distance bound. The sampled oracle alone does not establish
whole-face distance, all normals, editing behavior or full parity.

## Reproduction and provenance

From repository root:

    python3 truck-benchmarks/results/2026-10-16-g1-independent-oracle/run.py
    CARGO_BUILD_JOBS=12 cargo test --release -p truck-shapeops --test two_edge_blend_caps

The runner verifies every G1 input/result artifact against the committed clean
c8f6fc9c capture archive before sampling, records source/executable/shared-library
hashes, and checks input bytes again afterward. Build uses Wall/Wextra/Wpedantic.
Metadata, exact commands and all three sampling logs are stored beside this report.
Evidence archive: 30 byte-verified members, SHA256 d6423dd87c461962540744eebe04a084eeafaa686e827ac4f7006bc59c89fa93.
No fresh full capture was needed: production sources and geometry are unchanged.
No new kernel capability/fix is claimed; the existing regressions remain unchanged.
No full suite, Clippy or benchmark was redundantly rerun or claimed anew.

One initial diagnostic assumption rejected the input's negative extrusion axis.
The same unbounded support admits either axis sign; the probe now verifies
parallelism in either direction together with the unchanged explicit face-equation
and volume checks. The initial source/log are retained under diagnostics/ in the
archive. No existing test or tolerance was loosened.

The previously broad remaining-work label “full-solid two-edge attachment” needs
qualification: the existing two-edge FILLET cases are supported and tested.
Full-solid CHAMFER attachment beyond the shared-neighbor face-patch fix remains
an investigation. Other unresolved work includes H4's independent reference,
remaining discrepant cases, solver coverage, input gaps, continuity and latency.
