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

The `A4-dist_dist_sequence.json` and `A4-dist_angle_sequence.json` files are
byte-preserved automatic schema-2 capture 001 requests from the stable native
contour corpus (runtime 099ccb7e). Their input BREP SHA-256 is identical to the
equal-distance input: 677f27e9cdcf2b3548da3d0a84ec1913565392fc07b458569140eba431583e70.
They therefore reuse A4.step. Native volumes are respectively
0.5628218039597621 and 0.5625311455212756. These are native reference checks;
tests additionally verify the intended top/side setback assignment directly,
including reversed selection samples and flipped reference normal signs.

- `A4-dist_angle_sequence.json`: `602934ca3cc0fb6ea3f4f8dcfa507529c9a1d9a0934a46e913fc93cf5c774265`
- `A4-dist_dist_sequence.json`: `6e10e39c23e7251c64b40569a19b4a91e53efb335c834cd13cf56c441e077e02`
