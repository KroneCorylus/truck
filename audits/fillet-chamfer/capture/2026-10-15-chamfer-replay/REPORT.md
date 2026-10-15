# Full capture after native chamfer replay

Clean implementation/proof revision `99c956da1839bc2a170fb8b24a4e65aa8174f0f4`; frozen source/executable hashes
were checked before and after. All 157 selected native entries pass, producing
162 captures. The one new entry is bugs/moddata_1/bug203's initial chamfer.

106 valid constructions, consistent geometry, unchanged inputs, closed meshes,
prepared STEP and native-valid single-solid exports. 97 combined limited
comparison passes at .001. The remaining statuses are unchanged: 53 operation
failures, two shell-import failures, one incomplete capture. No unhandled intents,
panics or timeouts. All 105 prior successful result/comparison records are exactly
unchanged. No previously passing check regressed.

This expands captured coverage by one chamfer; it does not recover a previously
attempted fillet failure. The seven dimension-defined chamfer probes and one
fillet control remain separate from corpus counts. All 279 dedicated chamfer
fixtures remain blocked by missing data. bug203's later fillet failure check is
not replayed. Native distance comparisons are sampled, not certified global bounds.

`comparison-summary.json` lists differences; `results.json` records hashes and
commands. `artifacts.zip` contains byte-verified raw files and runtime.log.
SHA-256: `c321556746a17521226b13dc12b888b9f076f0d3b8d190dfbd103d0d1124cb02`.
Raw output: target/occt-capture-chamfer-replay.
Catalog checks and 19 audit plus five capture tests are in catalog-checks.txt.
