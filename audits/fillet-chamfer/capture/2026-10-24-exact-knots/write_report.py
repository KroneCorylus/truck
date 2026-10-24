"""Run from repository root after the exact-knot capture packager succeeds."""
from pathlib import Path
import hashlib
import html
import json
import shutil

root = Path('audits/fillet-chamfer')
name = '2026-10-24-exact-knots'
cap = root / 'capture' / name
checkpoint = root / 'checkpoints' / name
summary = json.loads((cap / 'comparison-summary.json').read_text())
manifest = json.loads((cap / 'results.json').read_text())
sha = lambda p: hashlib.sha256(Path(p).read_bytes()).hexdigest()
assert manifest['kernel_revision'] == '5a4e3bc666ffda4adb0ed03eeeb707c3068c1d79'
assert manifest['test_data_verified'] and not manifest['kernel_tracked_changes']
assert len(manifest['cases']) == 396 and summary['captures'] == 648
assert sha(cap / 'artifacts.zip') == manifest['artifact_archive_sha256']
assert not summary['unchanged_intent_regressions']
assert not summary['native_reference_changes']
assert not summary['scope_changes'] and not summary['seed_capture_changes']
assert not summary['new_limited_comparison_passes']
assert summary['unchanged_intent_records'] == summary['captures']
review = summary['successful_change_review']
assert review.get('provisional') is False
changes = summary['successful_record_changes']
assert {(r['case'], r['capture']): r['record_sha256'] for r in review['reviews']} == {
    (r['case'], r['capture']): r['record_sha256'] for r in changes
}
assert all(r['accepted'] and r['explanation'] and r['evidence'] for r in review['reviews'])
expected_gains = {
    ('chamfer/dist_dist_sequence/C1', '007'),
    ('chamfer/dist_dist_sequence/C1', '008'),
    ('chamfer/equal_dist_sequence/C1', '006'),
    ('chamfer/equal_dist_sequence/C1', '007'),
}
assert {(r['case'], r['capture']) for r in summary['status_changes']} == expected_gains
assert all(r['before'] == 'input-inconsistent' and r['after'] == 'operation-failed'
           for r in summary['status_changes'])
counts = summary['after_statuses']
constructed = counts['operation-succeeded']
passed = summary['limited_comparison_passes']
positive = summary['positive_solid_reference_count']
remaining = len(summary['unresolved_positive_solid_operations'])
assert remaining == positive - passed
exact = summary['exact_successful_results']
assert exact + len(changes) == constructed
no_solid = summary['input_scope_counts']['no-solid']
native_files = summary['identical_native_geometry_files_excluding_step_capture_timestamp']
matched_rejections = len(summary['matched_native_rejections'])
# Supplemental provenance: this records the present lockfile, not a retroactive
# claim that the original capture manifest or every historical build hashed it.
lock_hash = sha('Cargo.lock')
assert lock_hash == '58085733bd90cdb3ef2b8611cd443e22dae3281b715d5bdbca8707f3238d58bb'
lock_provenance = {
    'kind': 'supplemental-packaging-time-dependency-lock-snapshot',
    'path': 'dependency-Cargo.lock',
    'sha256': lock_hash,
    'capture_kernel_revision': manifest['kernel_revision'],
    'limitation': 'The capture manifest did not record Cargo.lock. This supplemental snapshot does not retrospectively attest every historical build dependency graph or change the archived capture manifest.',
}
paragraphs = [
    f"Complete serial capture of exact-knot implementation {manifest['kernel_revision']}: {len(manifest['cases'])} native cases, {summary['captures']} operations, {constructed} constructions and {passed} limited comparison passes. {remaining} of {positive} positive-solid operations remain unresolved. No newly resolved operation or new limited pass is claimed.",
    f"Compared with the committed mixed-corner capture, {exact} successful result/comparison records are exactly unchanged and {len(changes)} changed successful records have individual reviews bound to their exact record hashes. Their existing limited-comparison dispositions are preserved. The review retains material/volume, STEP, and bidirectional-distance evidence; it does not infer equivalence solely from small numerical deltas or assign an unproven unique cause to them.",
    f"All {summary['captures']} operation keys, raw input tasks and summarized tasks retain the same intent and native-contour selection scope. All {native_files} native geometry files are unchanged, excluding only generated FILE_NAME capture timestamps in STEP; BREP bytes are unchanged. The packager verifies source, replay, plugin/runtime, native harness, external data and archive member hashes. The capture uses one worker and a 120-second timeout.",
    "Four C1 inputs improve from inconsistent-input rejection to consistent input reaching construction, then fail at trim_faces: dist_dist_sequence/C1/007–008 and equal_dist_sequence/C1/006–007. These are input-consistency gains, not successful operations or resolved comparisons. Previously successful C1 outcomes remain unchanged.",
    f"Statuses: {json.dumps(counts, sort_keys=True)}. There are {constructed-passed} constructed comparison discrepancies. {no_solid} no-solid inputs already contain native shells/faces; their shell contracts remain unverified. {matched_rejections} rejection matches native. F1 remains a prior limited pass; H7 remains a constructed discrepancy against the unchanged 0.001 sampled-distance threshold.",
    "This evidence-only report reuses package D validation with its recorded source snapshots. Final post-inline checks: 367 dependency tests pass, one known pcurve_derivation precision failure and one existing ignore; 172 dependency doctests pass with three ignores; 81 mesher tests and 21 doctests pass with one test ignore; eight R10 and 76 replay tests pass. All-target Clippy and actions smoke pass with two existing test warnings. The complete 545-test/28-doctest downstream run precedes the ordinary inline annotation and is explicitly tied to that earlier source, not presented as a rerun on final source.",
    "The known pcurve_derivation assertion compares derivatives around magnitude 2.83e9 with an absolute difference around 1.4e-6. Its assertion and persisted counterexample are unchanged. This is not an all-green workspace claim. The full corpus executes the production replay but does not rerun those Rust suites.",
    "Fixed-core seven-sample alternating benchmarks retain about 3% finest-loft overhead: parent 111.3/113.6 ms, candidate 114.5/117.4 ms. Cap and sphere run averages are approximately unchanged. No zero-regression claim is made; exact sources, timings and earlier superseded evidence remain in the D implementation report.",
    "Exact knot equality, bounded checked-arithmetic diagnostics, IEEE infallible basis behavior, finite-search guards and the limited declared STEP spline-master preference are documented in the implementation report. These do not establish universal finite evaluation, panic freedom, native parity, or a global geometric-error bound. F5 and G2 prototypes are separate work and are not accepted capabilities of this capture. No consumer source or dependency pin changed; stopped E4 stress was not restarted.",
    f"Supplemental dependency provenance records the packaging-time Cargo.lock snapshot (SHA256 {lock_hash}). The original capture manifest did not hash Cargo.lock; this supplement does not retroactively certify every historical build graph.",
    f"Raw corpus archive SHA256: {manifest['artifact_archive_sha256']}.",
]
checkpoint.mkdir(parents=True, exist_ok=True)
shutil.copy2('Cargo.lock', cap / 'dependency-Cargo.lock')
(cap / 'dependency-lock-provenance.json').write_text(json.dumps(lock_provenance, indent=2) + '\n')
(cap / 'numeric-review.json').write_text(json.dumps(review, indent=2) + '\n')
shutil.copy2('target/package-exact-knot-capture-2026-10-24.py', cap / 'package_capture.py')
shutil.copy2(Path(__file__), cap / 'write_report.py')
(cap / 'REPORT.md').write_text('# Exact-knot complete corpus capture\n\n' + '\n\n'.join(paragraphs) + '\n\n'
    + '[Comparison summary](comparison-summary.json) · [Numeric review](numeric-review.json) · '
    + '[Dependency provenance](dependency-lock-provenance.json) · '
    + '[Implementation validation](../../../../truck-benchmarks/results/2026-10-24-exact-knots/REPORT.md)\n')
links = [
    ('../../capture/' + name + '/comparison-summary.json', 'Complete comparison'),
    ('../../capture/' + name + '/numeric-review.json', 'Changed-record review'),
    ('../../capture/' + name + '/artifacts.zip', 'Raw capture archive'),
    ('../../capture/' + name + '/dependency-lock-provenance.json', 'Supplemental dependency provenance'),
    ('../../../../truck-benchmarks/results/2026-10-24-exact-knots/REPORT.md', 'D implementation validation and performance'),
    ('../2026-10-24-mixed-curved-corners/g-discrepancy-review-2026-10-24.txt', 'Prior discrepancy classification'),
    ('../../TASK_TRACKER.md', 'Task tracker'),
]
style = 'html,body{margin:0;background:transparent;color:var(--foreground,#192b35);font:16px/1.6 system-ui,sans-serif}main{width:100%;min-width:0;box-sizing:border-box;margin:0;padding:0}h1{font-size:clamp(26px,4vw,42px);line-height:1.2}p,li{overflow-wrap:anywhere}a{color:var(--foreground,#075f91);text-decoration:underline}li{margin:.5em 0}.stats{font-size:1.2rem;font-weight:650;border-block:1px solid #8886;padding:1rem 0}'
page = '<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Exact-knot corpus checkpoint</title><style>' + style + '</style></head><body><main><p>Truck · 24 October 2026</p><h1>Exact-knot capture reconciled</h1><p class="stats">' + f'{constructed} construct · {passed} limited passes · {remaining} unresolved positive solids' + '</p>' + ''.join('<p>' + html.escape(p) + '</p>' for p in paragraphs) + '<h2>Evidence</h2><ul>' + ''.join('<li><a href="' + html.escape(url, quote=True) + '">' + html.escape(label) + '</a></li>' for url, label in links) + '</ul></main></body></html>'
(checkpoint / 'report.html').write_text(page)
coverage = json.loads((root / 'coverage.json').read_text())
coverage['scope']['execution'] = paragraphs[0] + f' {exact} successful records are exact; {len(changes)} changed successful records have hash-bound reviews. Four C1 input-consistency gains resolve no operations; {no_solid} shell inputs remain outside solid replay scope.'
coverage['scope']['latest_capture'] = {'implementation': manifest['kernel_revision'], 'report': 'capture/' + name + '/REPORT.md', 'summary': 'capture/' + name + '/comparison-summary.json'}
(root / 'coverage.json').write_text(json.dumps(coverage, indent=2) + '\n')
print(json.dumps({'constructions': constructed, 'limited_passes': passed, 'unresolved_positive_solids': remaining, 'exact_successful_records': exact, 'reviewed_changes': len(changes), 'report': str(checkpoint / 'report.html')}, indent=2))
