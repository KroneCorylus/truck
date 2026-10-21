# Native volume integration: clean capture

Implementation 2b125f413b273a6eb9542cf31ad74163546b9722.

Corrected the audit’s native volume measurements. An exact B-spline prism has volume 1.86295; both old audit producers reported 1.03219224. Gauss–Kronrod integration now respects surface spans and boundary-knot intervals, using a temporary copy. Original reference shapes, tolerances and historical archives are preserved.

All 27 native checks pass; ten failed on the parent. Controls cover an open face, box, cylinder, sphere, the exact spline prism, curved native references, rigid transforms and serialized input preservation. Native tools build cleanly; 13 capture and 22 catalog tests pass. Rust source hashes are unchanged, retaining the completed 444 shapeops, 115 stepio and 62 replay checks, Clippy and benchmark smoke. The known geometry precision failure and existing ignore remain explicit.

Clean capture: all 396 native cases pass across 648 operations. Counts remain 445 constructions and 415 limited comparison passes. All prior successful Truck result records and all non-volume comparison fields are exactly unchanged. 537 input and 547 native-result volume fields changed; 35 native-result changes exceed 1e-8 relative. These are measurement corrections, not new geometry capabilities.

Statuses: {"input-import-failed": 96, "operation-succeeded": 445, "operation-failed": 93, "unhandled-intent": 5, "input-inconsistent": 9}.

Archive SHA256: ad3323736adbd79fc14c6d8abfc0d77da56d0a6cf46dbd53f337894faaf10f8c.

No parity or speed claim. Finite sampling and requested integration tolerances do not certify global bounds. The unpublished trimming candidate and its failing tests are preserved in the investigation archive.
