"""Render all measured cases; retain losses and identify comparisons with different contracts."""
import json
from pathlib import Path
import re
import sys


def divan(path):
    results, parents = {}, []
    for line in Path(path).read_text().splitlines():
        # Tree indentation gives the group/action/argument path; the third column is median.
        match = re.match(r"^([│ ]*)[├╰]─ (\S+)\s*(.*)$", line)
        if not match:
            continue
        depth = len(match[1]) // 3
        parents = parents[:depth] + [match[2]]
        columns = [column.strip() for column in match[3].split("│")]
        if len(columns) >= 3 and columns[2]:
            value, unit = columns[2].split()
            results["/".join(parents)] = float(value) * {"ns": 1e-6, "µs": 1e-3, "ms": 1, "s": 1000}[unit]
    assert results, path
    return results


def comparison(key):
    _, action, *argument = key.split("/")
    if action in ["convert", "parse", "import", "export", "mesh_to_polygon"]:
        return None, "internal stage; no equivalent FreeCAD public API"
    if action == "circle_remove":
        return "circle_remove/kernel", ""
    if action in ["intersect_cylinders", "union_cylinders", "intersect_nurbs_cylinders"]:
        note = "Truck chord tolerance; FreeCAD uses OCCT geometric tolerances"
        if action == "intersect_nurbs_cylinders":
            note += "; NURBS representation differs (FreeCAD converts caps too)"
        return action, note
    note = {
        "loft": "different default spline interpolation; equivalent sections, not identical surfaces",
        "import_file": "same STEP bytes; Truck returns compressed topology, FreeCAD heals native B-rep",
        "draft_box": "FreeCAD includes document recompute",
        "hidden_lines": "Truck sampled occlusion; FreeCAD exact HLR",
        "plate": "absolute 0.01 mm deflection; independent mesh algorithms",
        "cylinder_tolerance": "absolute deflection; independent mesh algorithms",
    }.get(action, "")
    return "/".join([action] + argument), note


def main(directory):
    directory = Path(directory)
    before = divan(directory / "truck-before.txt")
    after = divan(directory / "truck-final.txt")
    freecad = {}
    text = (directory / "freecad-final.txt").read_text()
    assert "COMPLETE" in text and "FAILED" not in text, "incomplete FreeCAD run"
    for line in text.splitlines():
        if line.startswith("BENCH "):
            result = json.loads(line[6:])
            assert result["name"] not in freecad, "FreeCAD ran a case twice"
            freecad[result["name"]] = result["median_ms"]
    rows = ["| Operation | Truck before ms | Truck after ms | FreeCAD ms | FC / Truck | Comparison notes |",
            "|---|---:|---:|---:|---:|---|"]
    for key, value in after.items():
        reference, note = comparison(key)
        if reference is not None and reference not in freecad:
            raise ValueError(f"Missing FreeCAD counterpart: {reference}")
        fc = freecad.get(reference)
        old = f"{before[key]:.4g}" if key in before else "—"
        reference_time = f"{fc:.4g}" if fc is not None else "—"
        ratio = f"{fc / value:.2f}×" if fc is not None else "—"
        rows.append(f"| {key} | {old} | {value:.4g} | {reference_time} | {ratio} | {note} |")
    rows += ["", "A ratio above 1 favors Truck. Values below 1 are losses; small differences need repeated runs.", ""]
    app_before = {row["name"]: row["median_ms"] for row in map(json.loads, (directory / "app-before.jsonl").read_text().splitlines())}
    rows += ["| PiezaCad complete rebuild | Before ms | After ms | Speedup |", "|---|---:|---:|---:|"]
    for result in map(json.loads, (directory / "app-final.jsonl").read_text().splitlines()):
        old = app_before.get(result["name"])
        old_time = f"{old:.4g}" if old is not None else "—"
        ratio = f"{old / result['median_ms']:.2f}×" if old is not None else "—"
        rows.append(f"| {result['name']} | {old_time} | {result['median_ms']:.4g} | {ratio} |")
    (directory / "tables.md").write_text("\n".join(rows) + "\n")


if __name__ == "__main__":
    main(sys.argv[1])
