# Full public chamfer capture

Clean revision 8c1942c7eeb64451bf5734ad2bc202df113b52ed; explicit external fixture directory, with source,
executable and data hashes checked before/after.396 selected native entries pass,
producing648 independent operation captures.222 constructions are immutable,
geometrically consistent, closed meshes and native-valid single-solid STEP;
208 pass combined limited comparisons at0.001. All106 prior successful result/
comparison records are EXACTLY unchanged. No previous status/check/numeric regression.

The new239 native chamfer entries add486 captures,116 constructions and111 limited
comparison passes. Known native failures remain outside this passing-entry capture.
The unchanged native dedicated suite has239passed/22known-failure/18missing-data skips.
Per-operation sequence inputs come from native execution, not previous Truck output.

Remaining statuses:193 operation failures (including one confirmed native rejection),
124 unsupported intents (76 curved reference planes,48 nonplanar adjacent supports),
88 import failures (all inputs have native solids=0),21 inconsistent inputs in C1
sequence captures002..008 across three modes. No incomplete/unhandled process,
panic or timeout. Fourteen sampled discrepancies remain: nine old plus five
chamfer records. Tangent propagation is absent from the current seed-only capture;
A4/001's native builder expands one seed to three edges. That is a scope gap, not
an equivalence claim. Raw fixtures and source provenance are included in the archive.

7717 byte-verified archive members; SHA256 76e24a29cf27b3a7664f45bd5e868381d92406986260df6d1c61aecc30f112ba.
Raw directory target/occt-capture-public-chamfer-data. Catalog verification and
22 audit plus9 capture tests are in catalog-checks.txt. Rust/kernel/native plugin
unchanged in this group. No performance or full-parity claim.
