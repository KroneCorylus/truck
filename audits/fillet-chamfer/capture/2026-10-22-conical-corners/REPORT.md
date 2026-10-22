# Conical support corners: clean capture

Implementation 67ac586bbfe6ce098121f084946574ee5ed72df9.

Three selected edges on an outward cone and two planes now join an exact rolling sphere, toroidal cap rounds and fitted cone/radial-plane blends. Regular hyperbolic rolling centers handle axial terminations. Fitted contacts intersect remote caps before trim assembly; geometry, closure and trim checks remain.

467 release shapeops tests, 119 STEP tests, 74 replay tests, 13 capture tests, all-target Clippy and the single-thread benchmark smoke check pass. Native reference (8), volume (27) and distance (24) checks pass. Two regressions fail on the parent with identical sources and fixtures. Controls cover 48 procedural operations, eight oversized rejections and eight normalized imported selections. Four original-scale production replays pass their construction checks. One existing ignore, two existing test warnings and the known geometry precision failure remain explicit.

All 396 native cases pass across 648 captured operations. 477 constructions (+4) and 437 limited comparison passes (+0). All 473 previous successful result/comparison records are exactly unchanged, as are every input task, native reference and selection scope.

The initial four-worker capture encountered a kernel-confirmed out-of-memory kill in J3. A one-worker J3 retry exactly matched the previous result and native comparison with identical code, executables and inputs. The complete capture was then repeated with one worker; all 473 previous successes match exactly. The failed attempt, kernel log and serial retry are retained in capture-retry.zip with checked hashes.

C2, C6, C8 and D1 now construct but remain sampled-comparison discrepancies. C2 and C8 native corner samples differ from their exact radius-10 sphere by about 0.1655 and 4.8284; corresponding Truck samples lie on the sphere to numerical precision. Independent material integration converges within 2e-8 and checks raw and exported STEP solids. These observations do not explain every discrepancy. Canonical thresholds and statuses remain unchanged; no global-error bound or full parity is claimed.

Statuses: {"input-import-failed": 95, "operation-succeeded": 477, "operation-failed": 66, "input-inconsistent": 10}.

122 of 559 positive solid operations remain unresolved; 40 are constructed discrepancies.

Archive SHA256: 9a8cb1f2dfe4d335428b7b7cf4f83a6b4a01e1171472d2ab37dcadeff0680423.

No assertions, thresholds or replay fitting budgets changed. No parity, global-error or performance claim. No consumer/pin changes, push or PR.
