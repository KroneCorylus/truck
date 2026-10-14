# Independent closed fillet contours

Parent: `e1f87d42c9996c965633a3f0d20c2dab5def9acf`. Equal-radius edge selection previously rejected the two separated circular rims of a cylinder/cone stack with `NonPlanarFace`, although the wire solver already handled each rim separately. The new private fallback groups closed tangent components in original shell order, checks distinct support faces, applies existing wire fillets atomically, and rejects re-trimming a face already changed by another component. Final history maps original faces to the final solid and includes every generated face.

No public API, dependency or consumer change is required. The existing planar, local and additive routes retain priority. Open components, branching vertices, nontangent corners, shared support faces and shared modified trims remain outside the added scope. Intersections with distant faces remain outside the documented wire-solver scope; this is not a global collision detector.

## Parent-failing regression and independent geometry

`parent-regression.txt` records the new material/history/STEP regression failing on the parent with `NonPlanarFace`; the two negative regressions already pass there. No existing assertions or tolerances were changed.

The raw fixture is the native input from `feat/featrf/A4`. A5, A6 and B4 repeat the same initial blend before subsequent rib features: four captured occurrences of one geometric capability. Fixture bytes are unchanged. The test derives the volume from a radius-3, height-5 cylinder, a height-3 cone frustum (radii 2 and 1.5), and a radius-1, height-2 cylinder. Rounding the two reentrant rims by 0.3 adds the revolved profile integrals 0.16698658092757426 and 0.12948648959397593 to 56.25π, giving **177.01105983494742**. `independent-volume.py` reproduces this dimension-only calculation.

The regressions cover three transformed/scaled solids, three selection permutations with identical compressed output, the rolling radius and oriented normals, six original-to-final modified-face correspondences, four generated faces, closed meshes, material and STEP round trips. Negative cases reject shared-support contours and preserve the original input when an individual round cannot fit; a successful first individual round is also verified.

## Validation

- **960 passing tests, one unchanged baseline failure, five existing ignores**, across 153 targets in geometry, modeling, meshalgo, stepio and shapeops. The sole failure remains `pcurve_derivation`: −2829663063.761872 versus −2829663063.7618704. Parent proof: `../2026-10-13-cubic-endpoints/parent-precision-test.txt`.
- Release all-target Clippy, including benchmarks: passed.
- Single-thread action benchmark smoke: passed. No performance claim.
- Official release replay build: passed.
- All four targeted native captures construct, retain the input, remain geometrically consistent, close their meshes and export native-valid single solids.
- At replay tolerance 0.001, independent native integration of each Truck STEP gives **177.01108465949309**, absolute error **2.4824545675983245e-5**, approximately **0.14 ppm**. The independent material acceptance is one ppm. This measurement uses the replay tolerance, not the tighter 1e-6 kernel regression tolerance.
- Maximum native STEP edge/vertex tolerance: **7.3923650688958352e-6**.
- Sampled native-to-Truck distance: **3.5644795258947638e-6** over 1,473 samples. Reverse: **1.2247625619350748e-5** over 2,708 samples. Both are below 0.001. These are sampled checks, not certified global bounds.

`metadata.json` records commands, exit codes, source hashes, executable hashes, native provenance and all targeted results. All 24 native input/reference files were compared byte-for-byte with the preceding full capture. The target archive contains 49 byte-verified members. SHA-256: `da538e35aaca8e1f85965f1a5ccfa5d59b2472d3b8fb7cd1206f6d62c727b4a0`.

The subsequent full capture is recorded separately after committing this implementation; this targeted proof makes no overall parity claim.
