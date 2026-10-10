#!/usr/bin/env python3
"""Inventory OCCT blend tests without executing Tcl or claiming geometry parity.

python3 audits/fillet-chamfer/build_catalog.py --occt /path/to/OCCT --app ../PiezaCAD
Add --check to verify the checked-in catalog without writing files.
Only Python's standard library is required. See catalog.html for scope and usage.
"""

import argparse
from collections import Counter, defaultdict
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess

HERE = Path(__file__).resolve().parent
TRUCK = HERE.parent.parent
DEDICATED = {"blend", "chamfer", "fillet2d"}
RESERVED = {"begin", "end", "parse.rules", "cases.list", "grids.list"}
PACKAGES = {
    "BRepFilletAPI", "ChFi3d", "ChFi2d", "ChFiDS", "ChFiKPart", "BRepBlend",
    "Blend", "BlendFunc", "FilletSurf", "BRepFill", "BiTgte",
}
TOPIC = re.compile(r"fillet|chamf|blend|rollingball|chfi[23]d|buildevol|mkevol|updatevol", re.I)
API = re.compile(r"BRepFilletAPI_\w+|ChFi[23]d_\w+|BRepBlend_\w+|BiTgte_\w+|FilletSurf_\w+")
# Keep newlines/offsets while masking C++ comments and literals for brace matching.
CPP_TRIVIA = re.compile(r'//[^\n]*|/\*[\s\S]*?\*/|"(?:\\.|[^"\\])*"|\'(?:\\.|[^\'\\])*\'')
# DRAW registrations can have multiline help strings before __FILE__ and the callback.
REGISTRATION = re.compile(r'\b\w+\.Add\s*\(\s*"([\w-]+)"[\s\S]*?__FILE__\s*,\s*(\w+)\s*,[^;]*?\);')
# These tokens identify commands or wrappers, including legacy and OCAF interfaces.
CORE_COMMANDS = {
    "blend", "blend1", "fillet", "chamf", "chamf_throat",
    "chamf_throat_with_penetration", "mkevol", "updatevol", "buildevol",
    "bfuseblend", "bcutblend", "rollingball", "brollingball", "trollingball",
    "fillet2d", "chamfer2d", "chfi2d", "vblend", "AddFillet", "bossage",
    "continuityblend", "tolblend", "checkhist",
}
CONSTRUCTION = CORE_COMMANDS - {"continuityblend", "tolblend", "checkhist"}


def read(path):
    return path.read_text(encoding="utf-8", errors="replace")


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git(root, *args):
    return subprocess.check_output(["git", "-C", str(root), *args], text=True).strip()


def truck_revision(root):
    # Committing the audit must not make its own generated snapshot stale.
    return git(root, "log", "-1", "--format=%H", "--", ".", ":(exclude)audits/fillet-chamfer")


def source_ref(root, path, revision=None, line=1):
    rel = path.relative_to(root).as_posix()
    if revision:
        url = f"https://github.com/Open-Cascade-SAS/OCCT/blob/{revision}/{rel}#L{line}"
    else:
        url = Path(os.path.relpath(path, HERE)).as_posix() + f"#L{line}"
    return {"path": rel, "line": line, "url": url}


def commands_pattern(commands):
    return re.compile(r"(?<![\w:])(" + "|".join(re.escape(c) for c in sorted(commands, key=len, reverse=True)) + r")(?=[\s;\]}]|$)")


def function_body(text, name):
    masked = CPP_TRIVIA.sub(lambda m: re.sub(r"[^\n]", " ", m[0]), text)
    match = re.search(r"\b" + re.escape(name) + r"\s*\([^;{}]*\)\s*\{", masked)
    if not match:
        return "", 0
    start, depth = match.end(), 1
    for end in range(start, len(masked)):
        depth += (masked[end] == "{") - (masked[end] == "}")
        if depth == 0:
            return text[start:end], text.count("\n", 0, match.start()) + 1
    raise ValueError(f"Unclosed C++ callback: {name}")


def callback_context(text, callback, definitions, bodies):
    """Follow same-file helper calls; virtual/cross-file dispatch stays unproven."""
    pending, visited, contents = [callback], set(), []
    while pending:
        name = pending.pop()
        if name in visited:
            continue
        visited.add(name)
        if name not in bodies:
            bodies[name] = function_body(text, name)
        body, _ = bodies[name]
        contents.append(body)
        calls = set(re.findall(r"\b(\w+)\s*\(", body))
        pending.extend(sorted(calls & definitions - visited))
    return "\n".join(contents), sorted(visited - {callback})


def discover_commands(occt, revision):
    records = []
    for path in sorted((occt / "src").rglob("*.cxx")):
        text = read(path)
        if ".Add" not in text or not (API.search(text) or TOPIC.search(path.name) or any(f'"{c}"' in text for c in CORE_COMMANDS)):
            continue
        masked = CPP_TRIVIA.sub(lambda m: re.sub(r"[^\n]", " ", m[0]), text)
        # Find definition candidates, then follow only calls to names defined in this file.
        definitions = set(re.findall(r"\b(\w+)\s*\([^;{}]*\)\s*\{", masked)) - {"if", "for", "while", "switch", "catch"}
        bodies = {}
        # Global builder instances can be used by callbacks that omit the API type name.
        variables = set(re.findall(r"\bBRepFilletAPI_\w+\s*\*?\s+(\w+)\s*(?:[=(;])", text))
        without_comments = CPP_TRIVIA.sub(lambda m: re.sub(r"[^\n]", " ", m[0]) if m[0].startswith("/") else m[0], text)
        for match in REGISTRATION.finditer(without_comments):
            name, callback = match.groups()
            body, helpers = callback_context(text, callback, definitions, bodies)
            _, line = bodies[callback]
            signals = sorted(set(API.findall(body)))
            global_use = [v for v in variables if re.search(r"\b" + re.escape(v) + r"\s*(?:\.|->)", body)]
            if name not in CORE_COMMANDS and not signals and not global_use:
                continue
            records.append({
                "name": name, "callback": callback, "apis": signals,
                "same_file_helpers": helpers,
                "global_builder_references": sorted(global_use),
                "source": source_ref(occt, path, revision, line or text.count("\n", 0, match.start()) + 1),
                "evidence": "registered interface" if name in CORE_COMMANDS else "API use in callback; candidate relevance",
            })
    return sorted(records, key=lambda x: (x["name"], x["source"]["path"]))


def registered_cases(tests):
    """Expand grid aliases exactly once, as OCCT's TestCommands.tcl does."""
    result = []
    for grid_list in sorted(tests.glob("*/grids.list")):
        group = grid_list.parent.name
        for line in read(grid_list).splitlines():
            if "#" in line or not line.strip():
                continue
            fields = line.split()
            if len(fields) != 2 or not fields[0].isdigit():
                raise ValueError(f"Unrecognized grid declaration: {grid_list}: {line}")
            grid = fields[1]
            directory = tests / group / grid
            target = directory
            alias = directory / "cases.list"
            if alias.exists():
                target = (directory / read(alias).splitlines()[0].strip()).resolve()
            if not target.is_dir() or not target.is_relative_to(tests.resolve()):
                raise ValueError(f"Missing or external grid directory: {target}")
            helpers = [p for p in [tests / group / "begin", directory / "begin", directory / "end", tests / group / "end"] if p.is_file()]
            for case in sorted(target.iterdir()):
                if case.is_file() and case.name not in RESERVED and not case.name.startswith("."):
                    result.append((group, grid, case, helpers))
    return result


def numbered_lines(text, predicate):
    return [{"line": i, "text": line.strip()} for i, line in enumerate(text.splitlines(), 1) if predicate(line)]


def asset_references(text):
    # A Tcl expression is retained unresolved. No substitutions or scripts are evaluated.
    found = re.findall(r"\blocate_data_file\s+([^\]\n]+)", text)
    found += re.findall(r"^\s*set\s+chamf_shape\s+([^\n]+)", text, re.M)
    refs = []
    for raw in sorted(set(found)):
        value = raw.strip().strip('"{}')
        literal = bool(value) and not any(c in value for c in "$[] \t;")
        refs.append({"expression": raw.strip(), "name": value if literal else None})
    return refs


def tags_for(text, group, grid, commands):
    low = text.lower()
    tags = set()
    if group == "fillet2d" or commands & {"fillet2d", "chamfer2d", "chfi2d"} or "makefillet2d" in low:
        tags.add("2d")
    if group == "chamfer" or any("chamf" in c.lower() for c in commands):
        tags.add("chamfer")
    if group == "blend" or commands & {"blend", "blend1", "fillet", "AddFillet", "vblend", "bossage"}:
        tags.add("fillet")
    if commands & {"mkevol", "updatevol", "buildevol"} or "evol" in grid:
        tags.update(["fillet", "variable-radius"])
    if "throat" in low:
        tags.add("constant-throat")
    if "dist_angle" in grid:
        tags.add("distance-angle")
    if "dist_dist" in grid:
        tags.add("two-distances")
    if "equal_dist" in grid:
        tags.add("equal-distance")
    if "sequence" in grid:
        tags.add("sequential")
    if commands & {"bfuseblend", "bcutblend"}:
        tags.add("boolean-blend")
    if commands & {"rollingball", "brollingball", "trollingball"}:
        tags.add("face-rolling-ball")
    for tag, words in {
        "periodic-seam": ["seam", "periodic"], "degenerate": ["degenerat", "singular"],
        "freeform": ["bspline", "b-spline", "bezier", "nurbs"],
        "tangent": ["tangent", "tangency", "smooth edge"],
        "history": ["checkhist", "addfillet", "generated", "modified", "naming"],
        "tolerance": ["tolblend", "checkmaxtol", "tolerance"],
        "sphere": ["psphere", "sphere"], "cylinder": ["pcylinder", "cylinder"],
        "cone": ["pcone", " cone"], "torus": ["ptorus", "torus"],
        "self-intersection": ["self-intersect", "self intersect", "bopcheck"],
    }.items():
        if any(word in low for word in words):
            tags.add(tag)
    return sorted(tags)


def related_families(tags, families):
    """Keep 2D separate and require specific tags for specialized mechanisms."""
    tags = set(tags)
    related = []
    for family in families:
        family_tags = set(family.get("tags", []))
        if ("2d" in tags) != ("2d" in family_tags):
            continue
        operations = family_tags & {"fillet", "chamfer"}
        specifics = family_tags - {"fillet", "chamfer", "2d"}
        if operations and not operations & tags:
            continue
        if specifics and not specifics & tags:
            continue
        if family_tags & tags:
            related.append(family["id"])
    return related


def test_inventory(occt, revision, registry, families):
    tests = (occt / "tests").resolve()
    command_names = CORE_COMMANDS | {r["name"] for r in registry}
    command_rx = commands_pattern(command_names)
    assets = defaultdict(list)
    for path in sorted((occt / "data").rglob("*")):
        if path.is_file():
            assets[path.name].append(path.relative_to(occt).as_posix())
    cases, accounted = [], set()
    registrations = registered_cases(tests)
    for group, grid, case, helpers in registrations:
        text = read(case)
        text_commands = set(command_rx.findall(text))
        own_match = bool(TOPIC.search(text) or text_commands)
        helper_matches = [p for p in helpers if TOPIC.search(read(p)) or command_rx.search(read(p))]
        if group not in DEDICATED and not own_match and not helper_matches:
            continue
        accounted.add(case.resolve())
        direct = set(command_rx.findall("\n".join(l for l in text.splitlines() if not l.lstrip().startswith("#"))))
        relevance = "dedicated-suite" if group in DEDICATED else "command-candidate" if direct & (CONSTRUCTION | (command_names - CORE_COMMANDS)) else "reference-only"
        if not own_match and helper_matches:
            relevance = "inherited-helper-candidate"
        sources = [*[p for p in helpers if p.name == "begin"], case, *[p for p in helpers if p.name == "end"]]
        sources = list(dict.fromkeys(sources))
        markers, dependencies, checks = [], [], []
        for source in sources:
            content = read(source)
            for item in numbered_lines(content, lambda l: "TODO" in l or "REQUIRED" in l):
                markers.append({**item, "source": source.relative_to(occt).as_posix(), "conditional": "not evaluated"})
            for item in asset_references(content):
                available = assets.get(Path(item["name"]).name, []) if item["name"] else []
                dependencies.append({**item, "source": source.relative_to(occt).as_posix(), "checkout_candidates": available})
            for item in numbered_lines(content, lambda l: re.search(r"\b(checkshape|checkprops|checknbshapes|checkmaxtol|bopcheck|checkhist|checklength|checkarea)\b", l) is not None):
                checks.append({**item, "source": source.relative_to(occt).as_posix()})
        # A known per-case chamfer variable resolves the shared end script's dynamic reference.
        shape_match = re.search(r"^\s*set\s+chamf_shape\s+([^\n]+)", text, re.M)
        if shape_match:
            shape = shape_match[1].strip().strip('"{}')
            for item in dependencies:
                if item["expression"] == "$chamf_shape":
                    item["name"] = shape
                    item["checkout_candidates"] = assets.get(Path(shape).name, [])
        missing = sorted({d["name"] for d in dependencies if d["name"] and not d["checkout_candidates"]})
        unresolved = sorted({d["expression"] for d in dependencies if not d["name"]})
        tags = tags_for(text, group, grid, text_commands)
        family_ids = related_families(tags, families)
        cases.append({
            "id": f"{group}/{grid}/{case.name}", "group": group, "grid": grid,
            "relevance": relevance, "scope": "2d" if "2d" in tags else "3d-or-context",
            "source": source_ref(occt, case, revision), "sha256": sha(case),
            "helpers": [{**source_ref(occt, p, revision), "sha256": sha(p)} for p in helpers],
            "commands": sorted(text_commands),
            "operation_lines": numbered_lines(text, lambda l: not l.lstrip().startswith("#") and (command_rx.search(l) is not None or l.lstrip().startswith("set chamf_"))),
            "description_lines": numbered_lines(text, lambda l: bool(TOPIC.search(l)))[:8],
            "tags": tags, "family_hints": family_ids,
            "mapping_basis": "lexical tags; related mechanisms, not a verified Truck equivalent",
            "markers": markers, "checks": checks, "dependencies": dependencies,
            "missing_assets": missing, "unresolved_assets": unresolved,
            "dependency_status": "missing-in-checkout" if missing else "unresolved" if unresolved else "static-references-found" if dependencies else "no-static-reference",
            "source_commands": numbered_lines(text, lambda l: bool(re.search(r"\bsource\s+", l))),
            "occt_run": "not-run", "truck_run": "not-run", "comparison": "unverified",
            "reproduce": f"test {group} {grid} {case.name} -echo",
        })
    loose = []
    for path in sorted(tests.rglob("*")):
        if not path.is_file() or path.resolve() in accounted:
            continue
        text = read(path)
        if not (TOPIC.search(text) or command_rx.search(text)):
            continue
        loose.append({"source": source_ref(occt, path, revision), "sha256": sha(path), "kind": "harness" if path.name in RESERVED else "unregistered-or-reference", "matching_lines": numbered_lines(text, lambda l: bool(TOPIC.search(l) or command_rx.search(l)))[:10]})
    return sorted(cases, key=lambda x: x["id"]), loose, len(registrations)


def evidence_ref(spec, occt, app, revision):
    root = {"occt": occt, "truck": TRUCK, "app": app}[spec["repo"]]
    path = root / spec["path"]
    text = read(path)
    needle = spec["find"]
    if needle not in text:
        raise ValueError(f"Stale evidence: {path}: {needle}")
    line = text.count("\n", 0, text.index(needle)) + 1
    return {**spec, **source_ref(root, path, revision if spec["repo"] == "occt" else None, line), "sha256": sha(path)}


def source_inventory(occt, revision):
    files = []
    for package in sorted(PACKAGES):
        for path in sorted((occt / "src" / package).glob("*")):
            if path.suffix not in {".cxx", ".hxx", ".lxx", ".gxx"}:
                continue
            # BRepFill also contains unrelated sweep algorithms; retain only blend references.
            if package == "BRepFill" and not TOPIC.search(read(path)):
                continue
            files.append({"package": package, "source": source_ref(occt, path, revision), "sha256": sha(path), "lines": len(read(path).splitlines())})
    return files


def build(occt, app):
    occt, app = occt.resolve(), app.resolve()
    revision = git(occt, "rev-parse", "HEAD")
    coverage = json.loads(read(HERE / "coverage.json"))
    for family in coverage["families"]:
        family["evidence"] = [evidence_ref(e, occt, app, revision) for e in family["evidence"]]
    registry = discover_commands(occt, revision)
    cases, loose, all_count = test_inventory(occt, revision, registry, coverage["families"])
    source_files = source_inventory(occt, revision)
    if len({c["id"] for c in cases}) != len(cases):
        raise ValueError("Duplicate logical case IDs")
    counts = {key: dict(sorted(Counter(c[key] for c in cases).items())) for key in ["group", "relevance", "dependency_status", "comparison"]}
    counts.update({"cases": len(cases), "registered_cases_scanned": all_count, "distinct_case_files": len({c["source"]["path"] for c in cases}), "cases_with_markers": sum(bool(c["markers"]) for c in cases), "mechanisms": len(coverage["families"]), "source_files": len(source_files), "commands": len(registry), "unexecuted": len(cases)})
    counts["reference_topics"] = sum(f["truck_status"] == "reference-only" for f in coverage["families"])
    counts["capabilities"] = len(coverage["families"]) - counts["reference_topics"]
    return {
        "schema_version": 1,
        "snapshots": {
            "occt": {"revision": revision, "tag": git(occt, "describe", "--tags", "--always"), "tracked_changes": git(occt, "status", "--porcelain", "--untracked-files=no")},
            "truck": {"revision": truck_revision(TRUCK), "revision_scope": "Latest commit outside audits/fillet-chamfer; audit-only commits do not change the kernel baseline.", "tracked_changes_in_modeling": git(TRUCK, "status", "--porcelain", "--untracked-files=no", "--", "truck-shapeops", "truck-geometry", "truck-modeling")},
            "app": {"revision": git(app, "rev-parse", "HEAD"), "tracked_changes_in_core": git(app, "status", "--porcelain", "--untracked-files=no", "--", "cad-core")},
        },
        "scope": coverage["scope"], "status_definitions": coverage["status_definitions"],
        "validation_contract": coverage["validation_contract"], "variation_axes": coverage["variation_axes"],
        "counts": counts, "families": coverage["families"], "cases": cases,
        "commands": registry, "source_files": source_files, "other_references": loose,
        "discovery": {
            "rules": "All dedicated grids; all registered grids whose case or begin/end helpers match topic tokens or discovered commands; remaining textual matches retained separately.",
            "topic_pattern": TOPIC.pattern,
            "limits": ["Static inventory, not Tcl execution or geometric validation.", "Tags and family links are lexical hints, not a manual interpretation of each shape.", "Command discovery follows callbacks, same-file helpers and builder variables; arbitrary cross-file C++/Tcl indirection is not proven exhaustive.", "TODO/REQUIRED lines include conditional/platform-specific markers; conditions are preserved in linked source, not evaluated.", "Asset lookup checks only OCCT/data by filename; external CSF_TestDataPath locations and dynamically generated names may satisfy missing dependencies.", "All means the pinned source inventory under these explicit rules, not every possible geometric input or newer OCCT releases."],
        },
    }


def outputs(data):
    encoded = json.dumps(data, ensure_ascii=False, indent=2) + "\n"
    embedded = json.dumps(data, ensure_ascii=False, separators=(",", ":")).replace("<", "\\u003c").replace("\u2028", "\\u2028").replace("\u2029", "\\u2029")
    template = read(HERE / "viewer.html")
    if template.count("__CATALOG_DATA__") != 1:
        raise ValueError("Expected one catalog placeholder")
    return {"catalog.json": encoded, "catalog.html": template.replace("__CATALOG_DATA__", embedded)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--occt", type=Path, required=True)
    parser.add_argument("--app", type=Path, default=TRUCK.parent / "PiezaCAD")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    data = build(args.occt, args.app)
    stale = []
    for name, content in outputs(data).items():
        path = HERE / name
        if args.check:
            if not path.exists() or read(path) != content:
                stale.append(name)
        else:
            path.write_text(content, encoding="utf-8")
    print(json.dumps(data["counts"], indent=2))
    if stale:
        raise SystemExit("Catalog differs from these inputs: " + ", ".join(stale))
    print("Catalog verified." if args.check else "Catalog generated.")


if __name__ == "__main__":
    main()
