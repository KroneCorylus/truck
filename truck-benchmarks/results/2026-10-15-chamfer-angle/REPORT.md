# Planar distance-and-angle chamfer replay

Replay now converts a requested setback and angle on nonperpendicular planar
supports using oriented support normals. With corner angle beta and bevel angle
alpha, d1 = d0 sin(alpha) / sin(beta + alpha). Singular or nonpositive solutions
reject as harness errors before construction. Reference-plane normal sign and
face ordering do not exchange the requested setback. No kernel geometry or public
API changed; curved supports and broader collision behavior remain outside scope.

The previous nonperpendicular rejection assertion was replaced first with stronger
success tests for acute/obtuse wedges: both reference faces, both normal signs,
original/rigidly transformed frames, independent removed volume and bevel plane,
immutable input, closed-solid and STEP harnesses. Other rejection tests are unchanged.
A new negative test checks limiting and impossible angles at the obtuse corner.
All three new tests fail on parent 6b85a39f; the existing perpendicular test passes.
All 15 release replay tests now pass, as do all-target release Clippy including
benchmarks, action smoke, and five capture tests. Unchanged kernel suites were not
rerun wholesale; their preceding baseline remains 493 passes / 0 failures / 1 ignore.
No performance claim.

Eight native controls use a height-5 prism of (0,0), (1,0), (5,1), (0,1).
Acute corner beta=atan(1/4), alpha=30 degrees; obtuse beta=pi-atan(1/4), alpha=5.
The reference setback is 0.1. Independent volume is 15 - 5*d0*d1*sin(beta)/2.
Both reference faces and original/rotated/translated frames pass native validity,
single-solid STEP and material checks. Maximum sampled bidirectional distance:
1.1690547830822714e-11; maximum independent Truck STEP volume error: 1.6683543435647152e-11.
Native volumes independently match within 1e-6. Sampled distances are not global bounds.

These controls remain separate from corpus counts and do not recover any of the
279 missing-fixture native chamfer cases. The final native script uses physical
coordinates to verify transformed ordinals, which change under native sorting.
Raw evidence: 100 byte-verified archive members.
Archive SHA-256: 8a904bb5a1ed977f1f6d9427e1d530e999667d130494eb8a044581a440708a66.
Source/executable hashes, commands and native checks are in metadata.json.
A clean full capture follows the implementation/proof commit.
