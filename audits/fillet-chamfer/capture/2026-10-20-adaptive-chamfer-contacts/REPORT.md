# Adaptive chamfer contact fitting: clean capture

Implementation 81755598263323a478ac460c65684e10c96b5708.

Chamfer contact fitting now refines failing intervals after uniform sampling reaches its budget. The existing tolerance and sample budget are unchanged; local refinement is limited to eight rounds. Two parent-failing regressions cover twelve fits and 98,328 dense contact checks. Curved-cap C1 operations still fail downstream face trimming; this is a completed fitting improvement, not completed curved-cap solid support.

444 release shapeops checks, 115 stepio checks, 62 replay tests, 13 capture tests, all-target release Clippy and single-thread actions smoke pass. One existing shapeops ignore remains. Replay was built last. Unchanged core hashes retain prior validation, including the explicitly unresolved parent pcurve_derivation precision failure. No existing test was weakened or skipped.

Clean capture: 396 native entries pass; 648 operations. 445 constructions and 415 limited sampled comparison passes at the unchanged 0.001 threshold. 445 prior successful records are exactly unchanged; 2 unchanged-status failure diagnostics changed. No prior successful check regressed and no native reference property changed. Finite samples do not establish full parity or a global bound.

Statuses: {"input-import-failed": 96, "operation-succeeded": 445, "operation-failed": 93, "unhandled-intent": 5, "input-inconsistent": 9}.

Status changes: [].

Archive SHA256: 2c788594b90c0a7dd4ef04de394dffc58bd138437a05d086bc18be07f9e1336d.

Checkpoint complete. No consumer change, tolerance relaxation, parity claim or performance claim.
