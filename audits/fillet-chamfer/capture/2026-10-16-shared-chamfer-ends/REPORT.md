# Shared chamfer terminal faces: clean full capture

Clean runtime 71edbef189442dcbb3211718f2753c6a5c3b6b8c: all396native entries pass,648operation records.
All648requests/native reference properties and all231successful replay results
and complete comparisons are exactly unchanged. No status changes, numerical
changes or successful-check regressions.

Statuses: {"input-import-failed": 96, "operation-succeeded": 231, "operation-failed": 202, "unhandled-intent": 110, "input-inconsistent": 9}.
All231constructions preserve input and pass consistency, closed mesh, STEP
preparation and native valid-single-solid checks. The same220pass the combined
limited sampled comparison at0.001; eleven discrepancies remain.
The corpus has487complete native chamfer records and161complete fillet records.

The production fix assembles both end bridges together when a chamfer's open
ends share one terminal face. Its two parent-failing regressions pass in six
variants using independent material/cone/normal checks and shared solid/STEP
harnesses. Three additional native controls construct valid single-solid STEP
results with maxsample6.966905415601712e-9 and maxanalyticvolumeerror2.14e-8.
These newly supported controls do not change a case in the existing corpus.

Validation:391shapeops release passes,0failures,1existingignored;33replaypasses;
Clippy,actions smoke,finalreplaybuild pass. No existing assertion was loosened.
Implementation proof:truck-benchmarks/results/2026-10-16-shared-chamfer-ends/REPORT.md.
Archive:7744byte-verified members,SHA256 5d2fa3ceb8ac74d8295dc8a997d06f9d0bc55ba2c5d875a107805e9e6ed59422.
Finite sampling is not a global-distance or full-parity certificate.
No consumer/pin changes,push,PR or speed claim.
