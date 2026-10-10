#!/usr/bin/env python3
"""Build, validate and compare native Truck/OCCT kernels; no third-party Python packages."""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import signal
import statistics
import subprocess
import time

ROOT = Path(__file__).resolve().parents[2]
CASES = (
    [f"holes_{mode}/{n}" for mode in ("batch", "sequential") for n in (1, 10, 30, 100)]
    + [f"knurl/{n}" for n in (16, 64, 128)]
    + [f"thread/{n}" for n in (1, 4)]
    + [f"near_tangent/{gap}" for gap in ("0.01", "0.001")]
    + [f"thin_wall/{wall}" for wall in ("0.1", "0.01")]
    + ["fillet_bore/0.5"]
    + [f"mesh_plate/{n}" for n in (1, 100)]
)


def command(args, log, env=None, timeout=None):
    start = time.monotonic()
    with log.open("w") as output:
        process = subprocess.Popen(args, cwd=ROOT, env=env, stdout=output,
                                   stderr=subprocess.STDOUT, start_new_session=True)
        try:
            code = process.wait(timeout=timeout)
            status = "complete" if code == 0 else "failed"
        except subprocess.TimeoutExpired:
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            code = process.wait()
            status = "timeout"
    return dict(status=status, exit_code=code, wall_seconds=time.monotonic()-start,
                command=[str(a) for a in args], log=str(log.name))


def parse_result(text, samples, execution):
    result = dict(execution)
    phases = {}
    try:
        for line in text.splitlines():
            if line.startswith("RESULT "):
                record = json.loads(line[7:])
                phase = record["phase"]
                if phase in phases:
                    raise ValueError(f"duplicate {phase} record")
                phases[phase] = record
        result["timing"] = phases.get("timing")
        result["qa"] = phases.get("qa", {}).get("qa")
        if "failure" in phases:
            result["error"] = phases["failure"]["error"]
        if result["status"] != "complete":
            return result
        if "failure" in phases:
            raise ValueError("failure record with successful exit")
        times = phases["timing"]["samples_ms"]
        if len(times) != samples or not all(isinstance(t, (int, float)) and math.isfinite(t) and t > 0 for t in times):
            raise ValueError("missing, non-finite or incorrect sample count")
        qa = phases["qa"]["qa"]
        for key in ("volume", "expected_volume", "relative_volume_error", "qa_tolerance"):
            if not isinstance(qa[key], (int, float)) or not math.isfinite(qa[key]):
                raise ValueError(f"non-finite QA {key}")
        if qa["pass"] is not True:
            result["status"] = "invalid"
        elif (qa["volume"] <= 0 or qa["expected_volume"] <= 0 or qa["relative_volume_error"] > .002
              or qa.get("geometric") is not True or qa.get("mesh_closed") is not True
              or qa.get("topology", True) is not True or qa.get("solids", 1) != 1
              or not math.isclose(abs(qa["volume"]-qa["expected_volume"])/qa["expected_volume"],
                                  qa["relative_volume_error"],rel_tol=1e-8,abs_tol=1e-12)):
            raise ValueError("inconsistent QA pass")
        else:
            result["status"] = "pass"
    except (KeyError, ValueError, TypeError) as error:
        result["status"] = "protocol_error"
        result["error"] = str(error)
    return result


def summarize(records, cases, workers, rounds):
    rows = []
    indexed = {}
    for record in records:
        key = (record["case"], record["engine"], record["workers"], record["round"])
        if key in indexed:
            raise ValueError(f"duplicate measurement {key}")
        indexed[key] = record
    expected_keys = {(case, engine, worker, round_) for case in cases
                     for engine in ("truck", "occt") for worker in workers
                     for round_ in range(rounds+1)}
    if indexed.keys() != expected_keys:
        raise ValueError("missing or unexpected measurement coverage")
    for case in cases:
        for worker in workers:
            row = dict(case=case, workers=worker, ratio=None)
            for engine in ("truck", "occt"):
                runs = [indexed[case, engine, worker, r] for r in range(rounds+1)]
                failures = [r for r in runs if r["status"] != "pass"]
                if failures:
                    row[engine] = dict(status=failures[0]["status"])
                    continue
                if not all(math.isclose(r['qa']['expected_volume'],runs[0]['qa']['expected_volume'],rel_tol=1e-10) for r in runs):
                    raise ValueError(f"fixture changed across rounds: {case}/{engine}")
                timed = runs[1:]
                medians = [statistics.median(r["timing"]["samples_ms"]) for r in timed]
                values = sorted(t for r in timed for t in r["timing"]["samples_ms"])
                row[engine] = dict(status="pass", median_ms=statistics.median(medians),
                    round_medians_ms=medians, p95_ms=values[math.ceil(len(values)*.95)-1],
                    max_ms=max(values), samples=len(values),
                    peak_rss_kib=max(r["timing"]["peak_rss_kib"] or 0 for r in timed),
                    qa=timed[-1]["qa"])
            if all(row[e]["status"] == "pass" for e in ("truck", "occt")):
                a, b = row["truck"]["qa"], row["occt"]["qa"]
                if not math.isclose(a["expected_volume"], b["expected_volume"], rel_tol=1e-10):
                    raise ValueError(f"fixture mismatch: {case}")
                # Two individually acceptable errors can still disagree by twice the bound.
                row["agreement"] = abs(a["volume"]-b["volume"])/a["expected_volume"] <= .002
                if row["agreement"]:
                    row["ratio"] = row["occt"]["median_ms"]/row["truck"]["median_ms"]
            rows.append(row)
    return rows


def report(output, metadata, records):
    rows = summarize(records, metadata["cases"], metadata["workers"], metadata["rounds"])
    (output/"summary.json").write_text(json.dumps(rows, indent=2)+"\n")
    lines = ["# Truck versus direct OpenCascade", "",
        f"OCCT {metadata['occt_version']}; Truck `{metadata['revision']}` plus `source.patch` and `sources.sha256`. "
        f"{metadata['samples']} samples × {metadata['rounds']} rounds; CPUs {metadata['cpus']}.", "",
        "Native warmed operation latency in milliseconds. Setup, process startup, final-result destruction and QA are outside timers. "
        "Preflight and every timed process must pass native geometry checks, closed mesh and analytic volume within 0.2%. "
        "A failed preflight is retained and that engine/case is skipped in subsequent rounds. Ratios require both engines to pass and agree on volume. "
        "Ratios above 1 favor Truck; none is an overall speed claim.", "",
        f"Truck operation/mesh tolerance: {metadata['tolerance']} mm; QA deflection: {metadata['qa_tolerance']} mm, "
        "tightened for narrow intersections/walls, with a shared 1e-6 mm floor. OCCT uses native geometric tolerances, "
        f"additional fuzzy tolerance {metadata['fuzzy']} mm, and absolute mesh deflection matching Truck. These settings are different contracts.", "",
        "| Case | Workers | Truck median ms | OCCT median ms | OCCT / Truck | QA |",
        "|---|---:|---:|---:|---:|---|"]
    def median(data):
        return f"{data['median_ms']:.4g}" if data['status']=='pass' else "—"
    for row in rows:
        qa = "; ".join(f"{engine}: {row[engine]['status']}" for engine in ("truck", "occt") if row[engine]['status'] != "pass") or ("pass" if row.get("agreement") else "volume disagreement")
        ratio = f"{row['ratio']:.2f}×" if row['ratio'] is not None else "—"
        lines.append(f"| {row['case']} | {row['workers']} | {median(row['truck'])} | {median(row['occt'])} | {ratio} | {qa} |")
    lines += ["", "## Variation and memory", "",
        "P95 is the empirical nearest-rank value across raw samples, not a confidence interval. "
        "Peak RSS is the process high-water mark read **before postflight QA**, including fixture setup, warmup, "
        "libraries, thread pools and retained output. It is not isolated allocation cost. Thread setup includes STEP import/checking on OCCT.", "",
        "| Case | Workers | Engine | P95 ms | Max ms | Peak RSS MiB | Round medians ms |",
        "|---|---:|---|---:|---:|---:|---|"]
    for row in rows:
        for engine in ("truck", "occt"):
            data = row[engine]
            if data['status']=='pass':
                medians = ", ".join(f"{v:.4g}" for v in data['round_medians_ms'])
                lines.append(f"| {row['case']} | {row['workers']} | {engine} | {data['p95_ms']:.4g} | {data['max_ms']:.4g} | {data['peak_rss_kib']/1024:.2f} | {medians} |")
    lines += ["", "## Failures", ""]
    failures = [r for r in records if r['status'] not in ("pass", "skipped")]
    if not failures:
        lines += ["All recorded checks passed."]
    for record in failures:
        detail = record.get('error') or json.dumps(record.get('qa'), sort_keys=True)
        lines.append(f"- {record['case']}, {record['engine']}, {record['workers']} workers, round {record['round']}: "
                     f"**{record['status']}**. [Log]({record['log']}). {detail}")
    lines += ["", "## Scope", "",
        "- Knurl cases are straight axial triangular grooves, not diamond/helical knurling. Threads are external 60° grooves, 1.25 mm pitch, radius 4 mm, depth 0.4 mm; native Truck cutters are exported to STEP for OCCT outside timing. OCCT's standard STEP transfer may heal geometry; operands are not asserted to have identical topology.",
        "- Hole and meshing fixtures reuse the existing benchmark dimensions. Mesh results include fresh triangulation and triangle extraction; OCCT also copies topology without cached triangulation in the timed operation.",
        "- One worker disables OCCT Boolean/mesh parallelism. Multiple workers use OCCT's own thread pool with the stated limit; Truck uses Rayon. Both share the same CPU affinity. Fillet internal parallelism is API-dependent.",
        "- Mesh volume/closure and native checks are sanity gates, not proof of identical surfaces or universal modeling correctness. Truck's mesh checks and OCCT's native analyzer have different implementations. Topology counts may differ.",
        "- All engines and builds run sequentially. Even-numbered rounds reverse execution order. The active desktop, power policy, installed OCCT build and compiler differences remain sources of variation. No UI latency is measured.",
        "", "See `metadata.json`, `records.jsonl`, `summary.json`, `sources.sha256`, `source.patch` and raw logs. "
        f"Methodology and reproduction: [BENCHMARKS.md]({os.path.relpath(ROOT/'BENCHMARKS.md',output)}#direct-opencascade-comparison).", ""]
    (output/"REPORT.md").write_text("\n".join(lines))


def capture(args):
    return subprocess.check_output(args, cwd=ROOT, text=True, stderr=subprocess.STDOUT).strip()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--samples", type=int, default=15)
    parser.add_argument("--rounds", type=int, default=2)
    parser.add_argument("--workers", default="1,4")
    parser.add_argument("--cpus", default="0,1,2,3")
    parser.add_argument("--filter", default="", help="case substring")
    parser.add_argument("--timeout", type=float, default=180)
    parser.add_argument("--tolerance", type=float, default=.001)
    parser.add_argument("--qa-tolerance", type=float, default=.001)
    parser.add_argument("--fuzzy", type=float, default=0)
    parser.add_argument("--occt-include", type=Path)
    parser.add_argument("--occt-lib", type=Path)
    parser.add_argument("--report-only", action="store_true")
    args = parser.parse_args()
    output = args.output.resolve()
    if args.report_only:
        report(output, json.loads((output/"metadata.json").read_text()),
               [json.loads(line) for line in (output/"records.jsonl").read_text().splitlines()])
        return
    workers = [int(w) for w in args.workers.split(",")]
    cpus = [int(c) for c in args.cpus.split(",")]
    cases = [case for case in CASES if args.filter in case]
    if (not cases or args.samples<1 or args.rounds<1 or min(workers)<1 or len(set(workers))!=len(workers)
        or max(workers)>len(cpus) or not set(cpus)<=os.sched_getaffinity(0)
        or any(not math.isfinite(v) or v<=0 for v in (args.timeout,args.tolerance,args.qa_tolerance))
        or min(args.tolerance,args.qa_tolerance)<1e-6 or not math.isfinite(args.fuzzy) or args.fuzzy<0):
        parser.error("invalid cases, sample counts, workers, CPU affinity, timeout or tolerances")
    output.mkdir(parents=True, exist_ok=False)
    build = ROOT/"target/occt-comparison"
    cmake = ["cmake", "-S", "truck-benchmarks/occt", "-B", str(build), "-DCMAKE_BUILD_TYPE=Release"]
    if args.occt_include:
        cmake += [f"-DOCCT_INCLUDE_DIR={args.occt_include.resolve()}"]
    if args.occt_lib:
        cmake += [f"-DOCCT_LIBRARY_DIR={args.occt_lib.resolve()}"]
    build_commands = [("build-truck", ["cargo", "build", "--release", "-j2", "-p", "truck-benchmarks", "--example", "compare_occt"]),
                      ("configure-occt", cmake), ("build-occt", ["cmake", "--build", str(build), "-j2"])]
    for name, cmd in build_commands:
        result = command(cmd, output/f"{name}.log")
        if result["status"] != "complete":
            raise RuntimeError(f"{name} failed: {output/result['log']}")
    binaries = dict(truck=ROOT/"target/release/examples/compare_occt", occt=build/"compare_occt")
    fixtures = output/"fixtures"
    result = command([binaries['truck'], "--export-threads", fixtures], output/"prepare-fixtures.log", timeout=args.timeout)
    if result['status'] != 'complete':
        raise RuntimeError("thread fixture preparation failed; see prepare-fixtures.log")
    source_names = capture(["git","ls-files","--cached","--others","--exclude-standard","-z"]).split('\0')
    source_names.append('Cargo.lock')
    sources = {name: hashlib.sha256((ROOT/name).read_bytes()).hexdigest() for name in source_names
               if (ROOT/name).is_file() and (Path(name).suffix in ('.rs','.py','.cpp','.hxx') or Path(name).name in ('Cargo.toml','Cargo.lock','CMakeLists.txt'))}
    (output/"sources.sha256").write_text("".join(f"{digest}  {path}\n" for path,digest in sorted(sources.items())))
    patch = capture(["git", "diff", "HEAD", "--", "."])+"\n"
    untracked = capture(["git","ls-files","--others","--exclude-standard","-z"]).split('\0')
    for name in untracked:
        if name in sources:
            diff = subprocess.run(["git","diff","--no-index","--","/dev/null",name],cwd=ROOT,text=True,capture_output=True)
            if diff.returncode not in (0,1):
                raise RuntimeError(diff.stderr)
            patch += diff.stdout
    (output/"source.patch").write_text(patch)
    (output/"Cargo.lock").write_bytes((ROOT/"Cargo.lock").read_bytes())
    (output/"CMakeCache.txt").write_bytes((build/"CMakeCache.txt").read_bytes())
    (output/"git-status.txt").write_text(capture(["git", "status", "--short"])+"\n")
    libraries = capture(['ldd',binaries['occt']])
    library_hashes = {}
    for line in libraries.splitlines():
        fields = line.split()
        if len(fields)>=3 and fields[0].startswith('libTK') and fields[1]=='=>' and Path(fields[2]).is_file():
            library_hashes[fields[2]] = hashlib.sha256(Path(fields[2]).read_bytes()).hexdigest()
    metadata = dict(created_utc=time.strftime("%Y-%m-%dT%H:%M:%SZ",time.gmtime()),
        cases=cases, workers=workers, cpus=cpus, samples=args.samples, rounds=args.rounds,
        timeout_seconds=args.timeout, tolerance=args.tolerance, qa_tolerance=args.qa_tolerance, fuzzy=args.fuzzy,
        revision=capture(["git", "rev-parse", "HEAD"]), rustc=capture(["rustc", "-Vv"]),
        compiler=capture(["c++", "--version"]), platform=platform.platform(),
        lscpu=capture(["lscpu"]), occt_version=capture([binaries['occt'], "--version"]),
        build_commands=build_commands,
        occt_linked_libraries=libraries, occt_library_sha256=library_hashes,
        binaries={name:hashlib.sha256(path.read_bytes()).hexdigest() for name,path in binaries.items()},
        fixtures={p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in fixtures.iterdir()},
        cpu_governors={str(p):p.read_text().strip() for p in Path('/sys/devices/system/cpu').glob('cpu[0-9]*/cpufreq/scaling_governor')},
        environment={k:os.environ[k] for k in ('RUSTFLAGS','CXXFLAGS','CARGO_PROFILE_RELEASE_LTO') if k in os.environ})
    (output/"metadata.json").write_text(json.dumps(metadata,indent=2)+"\n")
    (output/".gitignore").write_text("!metadata.json\n!summary.json\n!Cargo.lock\n")
    records, blocked = [], set()
    jobs = [(case,engine,worker) for case in cases for worker in workers for engine in ('truck','occt')]
    with (output/"records.jsonl").open('w') as stream:
        for round_ in range(args.rounds+1):
            for case,engine,worker in (reversed(jobs) if round_>0 and round_%2==0 else jobs):
                key = (case,engine,worker)
                log = output/f"r{round_}-{engine}-{worker}t-{case.replace('/','-')}.log"
                if key in blocked:
                    record = dict(status="skipped", error="preflight did not pass", log=log.name)
                else:
                    count = 0 if round_==0 else args.samples
                    env = dict(os.environ, RAYON_NUM_THREADS=str(worker), OCCT_FUZZY=str(args.fuzzy))
                    cmd = ["taskset", "-c", args.cpus, binaries[engine], case, str(count),
                           str(args.tolerance), str(args.qa_tolerance), fixtures]
                    execution = command(cmd,log,env,args.timeout)
                    record = parse_result(log.read_text(),count,execution)
                    if round_==0 and record['status']!='pass':
                        blocked.add(key)
                record.update(case=case,engine=engine,workers=worker,round=round_)
                records.append(record)
                stream.write(json.dumps(record)+"\n")
                stream.flush()
                print(f"r{round_} {engine:5} {worker}t {case:24} {record['status']}",flush=True)
    report(output,metadata,records)
    print(f"Report: {output/'REPORT.md'}",flush=True)


if __name__ == '__main__':
    main()
