# Native chamfer contour fixture

Original OCCT 7.9.3 input for `chamfer/equal_dist_sequence/A4`, capture 001.
STEP and seed JSON are preserved byte-for-byte from the committed explicit
native contour proof in `truck-benchmarks/results/2026-10-15-tangent-chamfer-fit/native-evidence.zip`.
The expanded JSON is the independently inspected native builder's three-edge
contour. It was assembled explicitly for that earlier proof; it is not an
automatically captured schema-2 record. Tests construct schema-2 requests from
these separate pieces and retain the original seed. Native sampled comparison
and provenance remain in the earlier proof archive.

The geometry comes from the public OCCT fixture dataset recorded in
`audits/fillet-chamfer/runs/2026-10-15-public-chamfer-data/` and the pinned local
OCCT test inventory. Original native result volume: 0.5630665232959563; this is a
native reference check, not an independent analytic oracle. Independent analytic
kernel coverage remains in `tangent_chamfer_fit.rs`.

SHA-256:

- `A4.step`: `67c07f84f4e0d23c84528df85c6a29553e6868a3ecdc3438465dc46e5f23254c`
- `A4-seed.json`: `4763b2fa0e961edc135a789a670c5877d4585384cfcb796cae12ebae15fcefa1`
- `A4-expanded.json`: `a5a236e60a190ea9a9c6892bc664990c0e8437d5b57fd94f9b1f3ef2253d9579`
