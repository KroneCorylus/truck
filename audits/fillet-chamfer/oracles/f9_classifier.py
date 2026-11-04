#!/usr/bin/env python3
"""Mesh-free F9 classifier regression; run from any directory.

The original and 10x support charts must both classify the independently derived
material witnesses correctly. Exit 1 currently reproduces 28 OCCT classification
failures in the original artifact; it is not an expected-success/xfail mode.
No original fixture, classifier tolerance, or witness offset is modified.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    directory = Path(__file__).resolve().parent
    root = directory.parents[2]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--sdk", type=Path, default=root / "target/occt-sdk/usr")
    parser.add_argument("--out", type=Path, default=root / "target/f9-classifier-regression")
    args = parser.parse_args()
    sdk, out = args.sdk.resolve(), args.out.resolve()
    out.mkdir(parents=True, exist_ok=True)
    fixture = directory / "fixtures/f9-classification"
    original, source = fixture / "truck.step", fixture / "source.step"
    expected_hashes = {
        original: "cefed83c83d9dd6ece2430190d37dacb92a4cb0776bee44cb077b75e782b36f0",
        source: "65e74003d8ae682fa987e478f0ba716842d46a32de9677c723d36f563aec71d7",
    }
    if any(digest(path) != expected for path, expected in expected_hashes.items()):
        raise RuntimeError("archived F9 fixture identity changed")
    data = original.read_bytes()
    old = (b"#84 = B_SPLINE_CURVE_WITH_KNOTS('', 2, (#85, #86, #87), "
           b".UNSPECIFIED., .U., .U., (3, 3), (0.0, 1.0), .UNSPECIFIED.);")
    new = old.replace(b"(0.0, 1.0)", b"(0.0, 10.0)")
    if data.count(old) != 1:
        raise RuntimeError("expected exactly one archived parabolic basis declaration")
    chart = out / "chart10.step"
    chart.write_bytes(data.replace(old, new))
    assert chart.read_bytes().replace(new, old) == data
    binary = out / "f9_classifier"
    libraries = ["TKDESTEP", "TKXSBase", "TKShHealing", "TKTopAlgo", "TKGeomAlgo",
                 "TKBRep", "TKGeomBase", "TKG3d", "TKG2d", "TKMath", "TKernel"]
    command = ["g++", "-std=c++17", "-O2", "-Wall", "-Wextra", "-Werror",
               "-isystem", str(sdk / "include/opencascade"), str(directory / "f9_classifier.cpp"),
               "-L" + str(sdk / "lib64"), *("-l" + lib for lib in libraries), "-o", str(binary)]
    subprocess.run(command, check=True)
    env = dict(os.environ, LD_LIBRARY_PATH=str(sdk / "lib64") + ":" + os.environ.get("LD_LIBRARY_PATH", ""))
    linked = subprocess.run(["ldd", str(binary)], env=env, capture_output=True, text=True, check=True).stdout
    (out / "libraries.txt").write_text(linked)
    library_hashes = {}
    for line in linked.splitlines():
        if "=> /" in line:
            path = Path(line.split("=>", 1)[1].split()[0])
            if path.is_file():
                library_hashes[str(path)] = digest(path)
    with (out / "classification.jsonl").open("w") as stdout, (out / "stderr.log").open("w") as stderr:
        result = subprocess.run([str(binary), str(original), str(chart), str(source)],
                                env=env, stdout=stdout, stderr=stderr, timeout=45)
    rows = [json.loads(line) for line in (out / "classification.jsonl").read_text().splitlines()
            if line.startswith("{")]
    report = {
        "exit_code": result.returncode,
        "summary": rows[-1] if rows else None,
        "classifier_tolerance": 1e-7,
        "offset_capture_units": 0.0001,
        "control_change": "only #84 knot endpoint 1 -> 10; no spatial/trimming/tolerance changes",
        "compile_command": command,
        "hashes": {str(p): digest(p) for p in [original, source, chart, binary,
                                               directory / "f9_classifier.cpp", Path(__file__).resolve()]},
        "loaded_libraries": library_hashes,
        "limits": "Local sampled material test; no mesh, no native comparison pass, no global bound.",
    }
    (out / "results.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({"exit_code": result.returncode, "summary": report["summary"]}))
    return result.returncode


if __name__ == "__main__":
    raise SystemExit(main())
