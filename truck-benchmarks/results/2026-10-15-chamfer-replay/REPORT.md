# Native chamfer capture and physical replay

The capture previously wrapped only `blend`; chamfer requests were absent from
runtime evidence. It now wraps `chamf` in the native caller's scope, preserving
return values/error options and sharing a sequence counter with fillets.
The plugin retains the native parameter mode, distances/angle, physical edge
samples and analytic reference plane when available. Equal-distance parsing
uses the native command's `atof` semantics; other modes use `Draw::Atof`.

Replay uses the existing chamfer kernel APIs. It accepts equal distances,
per-edge planar two-distance assignments, and distance/angle requests between
perpendicular planar supports. Reference planes match exactly one adjacent
face, independently of ordinal IDs and normal sign. For perpendicular planes,
`d1 = d0*tan(angle)`. Unknown/ambiguous faces, unsupported curved reference
geometry and nonperpendicular angle requests remain explicit failures.
Duplicate physical edges with identical assignments deduplicate; conflicts reject.
Fillet selection and fallback behavior is preserved.

## Proof and validation

Two wrapper regressions fail on the parent; three replay success regressions
fail there as unhandled intent. New tests check both reference faces, reversed
plane normals, material and bevel normals, the common solid/STEP harness,
duplicate assignments and invalid/unsupported requests. Existing assertions
were not weakened. The runner selects only native-passing lexical `blend`/`chamf`
cases; missing dedicated fixtures are not reclassified.

All 12 release replay tests, five Python/Tcl capture tests, all-target release
Clippy including benchmarks, native plugin compilation and single-thread action
benchmark smoke pass. Kernel geometry is unchanged, so the previous 493-test
kernel-suite result remains the baseline; this run validates the changed tooling.
No performance improvement is claimed. The old geometry PCurve precision
assertion remains a separate open issue.

## Native evidence

Eight dimension-defined controls run through the real plugin: seven chamfers
and an interleaved fillet. They cover equal and unequal distances, either
reference face, a 30-degree angle and rigidly transformed boxes. They are
separate from the unchanged corpus. All have consistent geometry, unchanged
input, closed mesh and native-valid single-solid STEP. Maximum sampled distance
is 3.238e-11; independent native volume errors are below 1e-6.

The 10x20x30 box controls select a length-30 edge. Chamfer material is
`6000 - 30*d0*d1/2`; angle requests use `d1=d0*tan(30 degrees)`.
The fillet control uses `6000 - 30*4*(1-pi/4)`. These expectations derive from
dimensions, not native result volumes.

The existing corpus case `bugs/moddata_1/bug203` captures its initial chamfer
on a 100-cubed box, distance5/angle45 degrees. Expected material is
`1000000 - 100*5*5/2 = 998750`; native integration of Truck STEP differs by
3.493e-10. The later native fillet failure check is not replayed by this group.
All 279 dedicated chamfer cases still lack required data files. Synthetic
controls do not replace them and do not count toward recovered corpus coverage.

Native artifacts and probe source are retained. The byte-verified archive has
119 members, SHA-256 `c2c0d666d54f1f335bfafde92797bd39c183183e24a78bdede5a4f7a434c47ea`.
`metadata.json` records commands, frozen hashes, parameter captures, exact
expectations and sampled comparisons. A clean full capture after this commit
will verify the existing 161 captures and the newly selected chamfer case.
