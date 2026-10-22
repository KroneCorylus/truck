# Polar terminal fillet contacts: clean capture

Implementation aa770b1faba14a3cd815801a5b514f3b54727c1d.

Fillet contacts at a sphere parameter pole now recover through an analytic line/sphere intersection when the numerical solver fails. The fallback chooses the chord endpoint nearest the requested contact. Existing successful numerical contacts retain their behavior.

456 release shapeops tests, 119 STEP tests, 68 replay tests, 13 capture tests, all-target Clippy and the single-thread benchmark smoke check pass. Native reference (8), volume (27) and distance (24) checks pass. Two regressions fail on the parent with identical sources and fixtures. Controls cover eight successful variants, two collision rejections, 264 terminal-curve samples and two imported selections at two STEP fitting budgets. One existing ignore and the known geometry precision failure remain explicit.

All 396 native cases pass across 648 captured operations. 460 constructions (+1) and 427 limited comparison passes (+1). All 459 prior successes retain their checks; 458 result/comparison records are exactly unchanged. E1 also uses the analytic fallback: its sampled maximum distance and STEP material error decrease. Every input task, native reference and selection scope is unchanged.

New limited pass: blend/simple/E2/001.

Statuses: {"input-import-failed": 95, "operation-succeeded": 460, "operation-failed": 83, "input-inconsistent": 10}.

132 of 559 positive solid operations remain unresolved; 33 are constructed discrepancies.

Independent E1 section integration gives 3482832.6629430503; the updated STEP volume differs by 0.00013889 and maximum sampled distance is 2.085e-6. An additional unchanged-input, closed-solid/material/STEP probe is archived in E1-controls.zip.

Independent section integration gives E2 volume 3479626.174979871, within 1.76e-6 absolute volume of native.

Archive SHA256: ab76931107d07be4f972ba8e1890b503001d0c70046367eea1fe2cce811a44ba.

No assertions, thresholds or replay fitting budgets changed. No global-error, parity or performance claim. No consumer/pin changes, push or PR.
