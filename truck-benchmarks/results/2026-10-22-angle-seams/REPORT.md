# Distance-angle contacts across tangent support seams

A tilted planar cap over tangent cylindrical and planar walls produced two
cap-contact endpoints separated by 0.00010651494544464. The caller's tolerance
is 0.001, but strict chain trimming requires common endpoints. Distance-angle
chamfers now use the existing planar-cap continuation gate and reconcile those
endpoints on the shared plane. The existing constrained contact fit preserves
endpoint tangents and bounds displacement by the requested tolerance. Larger
gaps still fail. Strict topology and geometry checks remain unchanged.

Some imported contacts require localized refinement after endpoint constraints.
If the existing uniform fit exhausts its 257-sample budget, a local refinement
fallback bisects failing intervals, with the same 257-sample budget and the same
fit threshold min(tolerance, 1e-7). Existing successful uniform fits are unchanged.
The local fallback has a finite 16-refinement limit.

The new kernel regression checks six operations: both wire orientations,
identity and two transformed scales. Shared closed-solid, signed-material and
STEP checks use the independent native volume 0.7237600653608591, scaled cubically.
594 contact samples check analytic support equations, reference chords and
section angles. A tighter 1e-5 request still fails and preserves the input.
The new imported replay regression checks both selection orientations, all
three selected edges, unchanged inputs, consistent geometry, closed mesh,
prepared STEP, material and STEP round trip. Both new regressions fail at
trim_faces on parent 856900cd46366b2b9fd53b9a96e92019ebd55490 with byte-identical test sources and
no API adapter. No assertion or tolerance is weakened.

Final validation: 452 release shapeops checks (one existing ignore), 115 stepio,
66 replay, 13 capture, all-target Clippy and single-thread actions smoke pass.
Native reference 8, volume 27 and distance 24 checks pass. Unchanged geometry,
modeling and meshalgo evidence is reused by exact source hashes; the known
pcurve_derivation precision failure remains explicit. Replay is built last.

Fresh targeted chamfer/dist_angle/E5 constructs and passes all limited checks.
Native-to-Truck sampled distance: 4.8684277891693242e-05.
Truck-to-native sampled distance: 2.6628632492743933e-05.
The unchanged threshold is 0.001. This is sampled evidence, not a global bound.
The authoritative full capture follows the implementation commit.

Controls archive: 23 members, SHA256 51ce1e514965a58c682585292fab4bb4c24c726e23ac2ea6b05b2942ec2230f0.
See validation.json, parent.json, parent-tests.txt, parent-replay-tests.txt and
controls-summary.json. No consumer pin, push or PR changes.
