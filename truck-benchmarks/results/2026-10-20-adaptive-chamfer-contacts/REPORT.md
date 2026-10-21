# Adaptive chamfer contact fitting

Refine only failing parameter intervals after the existing uniform refinement reaches its sample budget. Preserve the original fit criterion, tolerance and sample budget; stop after at most eight local rounds. Previously successful uniform fits retain their original path.

Two regressions fail on parent 9687e33d; the existing interpolation control passes. Candidate tests cover two native curved-cap inputs, both directions and three setback pairs: twelve fits and 98,328 dense contact checks, ordered bounded samples, forward advance and immutable inputs. No existing assertion was weakened.

Full release validation: 444 shapeops passes (one existing ignore), 115 stepio passes, 62 replay tests, 13 capture tests, all-target release Clippy and single-thread actions benchmark smoke. Replay built last. Unchanged geometry/modeling/meshalgo source hashes reuse earlier evidence; the independently reproduced pcurve_derivation precision failure remains unresolved.

The native fixtures are byte-identical to the prior clean unequal tilted-cap capture; this checkpoint does not claim new native reference runs or completed curved-cap solids. Original uniform ladders and even 257 uniform samples fail near local knot transitions. Adaptive fitting passes within the same budget. Direct pipeline probes advance to trim_faces and still fail there, preserving their inputs. Scratch diagnostic fitting is illustrative; validation.json identifies the exact final bounded production source.

The subsequent clean full capture and HTML checkpoint report record any corpus changes. No global error bound, full parity or performance improvement is claimed. The previous interrupted validation is retained separately; the resumed validation certifies this checkpoint.
