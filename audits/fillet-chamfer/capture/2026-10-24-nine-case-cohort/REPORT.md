# Nine-case additional capture cohort

Separate nine-case instrumented cohort: 9 actual captured operations, 0 constructions and 0 limited comparison passes. 1 positive-solid operations remain unresolved in this cohort.

Native statuses: {'passed': 9}. Truck statuses: {'operation-failed': 1, 'input-import-failed': 8}. Cases without intercepted operations: []. Construction alone is not a pass; the unchanged sampled-distance threshold is 0.001.

The cohort has 1 positive-solid native reference and 8 native no-solid inputs. D2 has a valid native solid but Truck rejects unsupported topology at validate_input. The other eight native inputs/results are valid zero-solid shapes and remain outside solid replay scope; they are not solid modeling successes or native rejections.

The complete 396-case/648-operation exact-knot baseline remains immutable and remains the latest complete capture. This nine-case manifest does not replace it. A separate reviewed index combines both immutable cohorts as 405 cases / 657 operations, with 438 limited passes and 122 unresolved positive-solid operations; this is not a fresh 405-case execution.

Source, replay, canonical plugin, runtime, harness and dependency-lock hashes were verified against the existing D capture/launch. Native source and external fixture hashes match the native-only retry. That retry did not archive intercepted native input/result geometry, so geometry equality to it is not claimed; this archive preserves all new captured input/result artifacts and verifies every archived byte.

Root evidence revision 7c2ce2865070a56b25f1387c6121db8218b994f9; verified implementation revision 5a4e3bc666ffda4adb0ed03eeeb707c3068c1d79. No kernel implementation changed. No global geometric-error bound, universal parity, or newly resolved previous ledger row is inferred.

Archive SHA256: b24995efa4adccdd332ba924775de8d60a7ac0228d6aa00700178be99cc4894b.

[Per-operation checks](cohort-summary.json) · [Complete baseline](../2026-10-24-exact-knots/REPORT.md)

[Combined evidence index](combined-evidence-index.json) · [Native-intent review](combined-evidence-review.txt)
