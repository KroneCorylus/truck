# Full bounded STEP edge capture

Clean revision 3795c05123186de9475df2efba934796b7e7dd3f. All396 native entries pass, yielding648
operation captures. All222 previous successful result/comparison records are
EXACTLY unchanged: immutable inputs, consistent geometry, closed meshes and
native-valid single-solid STEP.208 combined limited comparisons still pass at.001.
There are no previous successful status/check/numeric regressions.

Four C1 inputs now pass consistency: equal-distance sequence002/003 reach blend
handling and reject immutably; two-distance sequence002/003 reach unsupported
curved-reference intent. Eight formerly inconsistent inputs now reject during
import: seven angle-mode cases have unmatched bounded endpoints; equal-distance
006 reaches unsupported modeling conversion. Nine inputs remain inconsistent.
New counts:222 constructions,195 operation failures,126 unhandled intents,
96 import failures,9 inconsistent inputs. No incomplete/panic/timeout.
No new successful chamfer constructions or parity promotion.

The 14 prior sampled discrepancies remain. Sequence records use native input
for every step, not previous Truck outputs. Native tangent propagation is still
absent from seed-only capture; a complete A4 contour exposes an independent
chamfer fit defect. Closed shells, curved reference support, four missing fixture
names and the prior PCurve precision assertion remain open.

7717 byte-verified archive members; SHA256 e63110d430beb3567bea3ad6f23f3c364b52068d77f1744ecea47e7e7724c548.
Raw target/occt-capture-bounded-step-edges. Implementation proof includes498
release tests passing,0 failing,1 existing ignore; clean all-target Clippy,
action smoke and15 replay tests. No speed claim. Catalog checks and Python
runner tests are recorded separately in catalog-checks.txt.
