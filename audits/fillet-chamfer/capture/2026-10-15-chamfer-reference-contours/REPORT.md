# Chamfer reference propagation: clean full capture

Clean runtime revision 48a1032fc769831ad0754c6639015ff77d59f505. All 396 native entries pass, yielding 648 captures:
487 complete native chamfer contour records and 161 seed-only fillet records.
All captured intent and native reference properties are exactly unchanged from
the preceding complete-contour checkpoint. No prior successful check regresses.

Statuses: {"input-import-failed": 96, "operation-succeeded": 231, "operation-failed": 202, "unhandled-intent": 110, "input-inconsistent": 9}.
All 231 constructions preserve input, geometric consistency, closed
meshes, prepared STEP and native single-solid validity. 220 pass combined
limited sampled comparisons at 0.001; 11 retain differences.
221 prior successful results and comparisons are exactly unchanged;
0 have recorded numerical changes. See comparison-summary.json for
all status transitions and individual checks.

The replay matches a planar seed reference once, then propagates its side along
oriented contour traversal. Consistent unequal setback pairs reach the existing
wire API. Planar supports and perpendicular plane/cylinder rims are supported;
other curved pairs and curved seed references remain explicitly unsupported.
Kernel source, physical matching and STEP fitting tolerances are unchanged.
Two native parent-failing regressions cover contour reversal, reference normal
sign, exact per-face setback assignment, immutable input and shared solid/STEP
checks. All 27 replay tests, Clippy and action smoke pass. Targeted native A4
unequal/angle contours select all three edges, return valid single-solid STEP,
agree with native volume within 5e-12 and have maximum sampled difference 2.23e-8.

Archive: 7744 byte-verified members; SHA256 aa1d106710b8ee87114683f35105d36047c8f072dbc478f55fd4e557dd2b6a8c.
These are finite sampled checks, not certified global distance or full parity.
Native sequence steps supply independent inputs, not chained Truck outputs.
Fillet contour/radius-law capture, remaining input and solver limitations,
sampled discrepancies, continuity/editing behavior and latency remain open.
No consumer/pin change, push or PR.
