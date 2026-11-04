"""Local triangle witnesses in four fresh Truck STEP exports; no global material claim.

Usage: python a2_triangle_material.py EVIDENCE_DIR
Build material-reader there from a2_triangle_material.cpp against the retained OCCT SDK.
"""

import hashlib
import json
import math
from pathlib import Path
import subprocess
import sys

out = Path(sys.argv[1]).resolve()
reader = out / "material-reader"
results = []
for mode in ["dist_dist", "dist_angle"]:
    a = 0.1 if mode == "dist_dist" else 0.2 / math.sqrt(3)
    b = 0.2
    contacts = [(0, b, b), (a, a, 0), (a, 0, b)]
    normal = [b * b, a * b, a * a]
    length = math.sqrt(sum(v * v for v in normal))
    normal = [v / length for v in normal]
    for case in ["A7", "A9"]:
        step = (
            out
            / "scoped/chamfer"
            / (mode + "_complex")
            / case
            / "capture/001.truck.step"
        )
        probes = []
        for far in [False, True]:
            for weights in [
                (1 / 3,) * 3,
                (0.6, 0.2, 0.2),
                (0.2, 0.6, 0.2),
                (0.2, 0.2, 0.6),
            ]:
                center = [
                    sum(weights[i] * contacts[i][j] for i in range(3)) for j in range(3)
                ]
                for offset, expected in [(-0.001, 1), (0, 2), (0.001, 0)]:
                    p = [center[i] + offset * normal[i] for i in range(3)]
                    if far:
                        p[0] = 1 - p[0]
                    text = subprocess.check_output(
                        [str(reader), str(step), *map(str, p)], text=True
                    )
                    state = int(text.split("STATE ")[-1])
                    probes.append(
                        dict(
                            point=p,
                            offset=offset,
                            expected=expected,
                            state=state,
                            passed=state == expected,
                        )
                    )
        results.append(
            dict(
                mode=mode,
                case=case,
                step_sha256=hashlib.sha256(step.read_bytes()).hexdigest(),
                probes=probes,
            )
        )
(out / "material.json").write_text(json.dumps(results, indent=2))
passed = sum(p["passed"] for r in results for p in r["probes"])
print(f"{passed}/96 local triangle material witnesses passed")
assert passed == 96
