# Stored native tolerances: clean full capture

Clean runtime c8f6fc9cc51d6d50a330323cee0230b7acafa505: 396 native entries passed, 648 operation records.
All 648 requests and native reference properties are exactly unchanged after
excluding the added tolerance metadata. All 231 successful replay results and
comparisons remain exactly unchanged. No status changes or regressions.

Statuses: {"input-import-failed": 96, "operation-succeeded": 231, "operation-failed": 202, "unhandled-intent": 110, "input-inconsistent": 9}.
All 231 constructions preserve input and pass geometry consistency, closed mesh,
STEP preparation and native valid-single-solid checks. 220 also pass the combined
limited sampled comparison at 0.001; the same 11 discrepancies remain unresolved.

Stored tolerance maxima are now recorded separately for faces, edges and
vertices. Of 647 available native result shapes, 28 have a maximum above 0.001;
7 input shapes also exceed it. None of 231 imported Truck STEP results exceeds it.
The rejected native operation has no result shape and is not counted as measured.
Only H4 and tolblend_simple/A1 are above threshold among the 231 successful
Truck requests. Both were already sampled discrepancies. Every one of the 220
limited passes has native stored tolerances at or below the threshold.

H4 native maximum: 22.9741069520462; Truck STEP: 0.00001.
tolblend_simple/A1 native maximum: 0.00235108915834048.
These stored budgets do not measure actual error. A controlled box with face
0.05, edge 0.1 and vertex 0.2 tolerances remains exact against STEP, with zero
sampled distance and independently known volume 6000. The parent regression
fails because these fields were absent. No validity flag, threshold, geometry
or parity verdict was changed. H4 needs an independent geometry oracle; G1's
small native tolerance budget does not explain its remaining discrepancy.

13 capture and 22 audit tests, C++ warnings build, all-target release Clippy,
action smoke and final replay build pass. Rust replay/kernel source is unchanged;
prior 33 replay and 504 kernel passes were not rerun or claimed anew.
Implementation proof: truck-benchmarks/results/2026-10-15-native-tolerances/REPORT.md.
Full archive: 7744 byte-verified members; SHA256 b2d83e50c5c4a44992aa5bc3423d3cb9dbe9dd075358ee537c09dba87ee1ae01.
Finite sampled comparisons do not certify global distance or full parity.
No consumer/pin change, push, PR or performance claim.
