"""Checks against omissions and false coverage claims in the static inventory."""

import json
from pathlib import Path
import tempfile
import unittest
import zipfile

import build_catalog as audit


class CatalogTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)

    def put(self, path, text):
        file = self.root / path
        file.parent.mkdir(parents=True, exist_ok=True)
        file.write_text(text)
        return file

    def test_aliases_preserve_modes_helpers_and_conditional_expectations(self):
        self.put("tests/chamfer/grids.list", "001 equal_dist\n002 dist_angle\n")
        for grid in ["equal_dist", "dist_angle"]:
            self.put(f"tests/chamfer/{grid}/cases.list", "../data\n")
            self.put(f"tests/chamfer/{grid}/begin", f"set group {grid}\n")
        self.put("tests/chamfer/data/A1", 'set chamf_shape sample.brep\nset chamf_equal_dist [list "1"]\n')
        self.put("tests/chamfer/end", 'if {$group eq "dist_angle"} { puts "TODO BUG Windows: failed" }\nrestore [locate_data_file $chamf_shape] s\ncheckshape s\n')
        self.put("data/brep/sample.brep", "fixture")
        cases, _, _ = audit.test_inventory(self.root, "abc", [], [])
        self.assertEqual([c["id"] for c in cases], ["chamfer/dist_angle/A1", "chamfer/equal_dist/A1"])
        self.assertEqual(len({c["sha256"] for c in cases}), 1)
        for case in cases:
            self.assertEqual(case["comparison"], "unverified")
            self.assertEqual(case["occt_run"], "not-run")
            self.assertEqual(case["markers"][0]["conditional"], "not evaluated")
            self.assertEqual(case["missing_assets"], [])
            self.assertEqual(case["unresolved_assets"], [])
            self.assertEqual(case["checks"][0]["text"], "checkshape s")

    def test_cpp_hidden_command_through_helper_and_literal_braces(self):
        self.put("src/QABugs/test.cxx", '''
static int helper() { const char* s="}"; /* } */ BRepFilletAPI_MakeFillet blend(shape); return 0; }
static int HiddenQA() { return helper(); }
void registerCommands() {
  // di.Add("Commented", "ignored", __FILE__, HiddenQA, group);
  di.Add("HiddenCase", "help", __FILE__, HiddenQA, group);
}
''')
        commands = audit.discover_commands(self.root, "abc")
        self.assertEqual([c["name"] for c in commands], ["HiddenCase"])
        self.assertIn("helper", commands[0]["same_file_helpers"])
        self.put("tests/bugs/grids.list", "001 regression\n")
        self.put("tests/bugs/regression/bug1", "HiddenCase result\n")
        cases, _, _ = audit.test_inventory(self.root, "abc", commands, [])
        self.assertEqual(cases[0]["id"], "bugs/regression/bug1")
        self.assertEqual(cases[0]["relevance"], "command-candidate")

    def test_reference_only_missing_dynamic_and_unregistered_sources_retained(self):
        self.put("tests/misc/grids.list", "001 refs\n")
        self.put("tests/misc/refs/a", '# fillet reference only\nrestore [locate_data_file missing.brep] s\nrestore [locate_data_file $dynamic] t\n')
        self.put("tests/misc/loose", "# chamfer historical note\n")
        cases, loose, count = audit.test_inventory(self.root, "abc", [], [])
        self.assertEqual(count, 1)
        self.assertEqual(cases[0]["relevance"], "reference-only")
        self.assertEqual(cases[0]["missing_assets"], ["missing.brep"])
        self.assertEqual(cases[0]["unresolved_assets"], ["$dynamic"])
        self.assertEqual(loose[0]["source"]["path"], "tests/misc/loose")

    def test_inherited_helper_can_make_an_otherwise_opaque_case_relevant(self):
        self.put("tests/custom/grids.list", "001 fixtures\n")
        self.put("tests/custom/fixtures/a", "set radius 1\n")
        self.put("tests/custom/fixtures/end", "blend result s $radius e\n")
        cases, _, _ = audit.test_inventory(self.root, "abc", [], [])
        self.assertEqual(cases[0]["relevance"], "inherited-helper-candidate")

    def test_missing_grid_is_an_error_instead_of_silent_omission(self):
        self.put("tests/blend/grids.list", "001 missing\n")
        with self.assertRaisesRegex(ValueError, "Missing or external"):
            audit.registered_cases((self.root / "tests").resolve())

    def test_generic_fillet_reference_does_not_imply_curved_or_2d_evidence(self):
        families = [
            {"id": "plain", "tags": ["fillet"]},
            {"id": "curved", "tags": ["fillet", "freeform"]},
            {"id": "sketch", "tags": ["2d"]},
            {"id": "curved-chamfer", "tags": ["chamfer", "cylinder", "freeform"]},
        ]
        self.assertEqual(audit.related_families(["fillet"], families), ["plain"])
        self.assertEqual(audit.related_families(["fillet", "2d"], families), ["sketch"])
        self.assertEqual(audit.related_families(["chamfer", "cylinder"], families), ["curved-chamfer"])

    def test_embedded_catalog_cannot_close_its_script_element(self):
        result = audit.outputs({"payload": "</script><script>alert(1)</script>"})
        self.assertNotIn("</script><script>alert(1)", result["catalog.html"])
        self.assertEqual(json.loads(result["catalog.json"])["payload"], "</script><script>alert(1)</script>")

    def test_committing_the_audit_does_not_change_the_kernel_baseline(self):
        audit.git(self.root, "init", "-q")
        def commit():
            audit.git(self.root, "add", ".")
            audit.git(self.root, "-c", "user.name=Audit Test", "-c", "user.email=audit@example.invalid", "-c", "commit.gpgsign=false", "commit", "-qm", "fixture")
            return audit.git(self.root, "rev-parse", "HEAD")
        self.put("truck-modeling/src/geometry.rs", "first")
        baseline = commit()
        self.put("audits/fillet-chamfer/catalog.json", "snapshot")
        commit()
        self.assertEqual(audit.truck_revision(self.root), baseline)
        self.put("truck-modeling/src/geometry.rs", "second")
        changed = commit()
        self.assertNotEqual(changed, baseline)
        self.assertEqual(audit.truck_revision(self.root), changed)

    def execution_fixture(self):
        source = self.put('occt/tests/bugs/grid/a', 'blend result s 1 e\n')
        log = self.put('runs/first/bugs/grid/a/console.log', 'CASE bugs grid a: BAD (known problem)\n')
        case = dict(id='bugs/grid/a', source={'path': 'tests/bugs/grid/a'},
                    sha256=audit.sha(source), helpers=[], occt_run='not-run',
                    truck_run='not-run', comparison='unverified')
        result = dict(id=case['id'], status='known-failure', source_sha256={case['source']['path']: case['sha256']},
                      artifacts={'bugs/grid/a/console.log': audit.sha(log)},
                      comparison='unverified', truck_run='not-run')
        self.put('runs/first/results.json', json.dumps(dict(schema_version=1, cases=[result], runtime='OCCT')))
        return case, source, log

    def test_native_run_does_not_promote_parity_or_known_failure_to_pass(self):
        case, _, _ = self.execution_fixture()
        runs = audit.apply_runs([case], self.root / 'occt', self.root / 'runs')
        self.assertEqual(case['occt_run'], 'known-failure')
        self.assertEqual(case['truck_run'], 'not-run')
        self.assertEqual(case['comparison'], 'unverified')
        self.assertEqual(len(runs), 1)

    def test_execution_rejects_changed_source_and_missing_log(self):
        case, source, log = self.execution_fixture()
        log.unlink()
        with self.assertRaisesRegex(ValueError, 'artifact'):
            audit.apply_runs([case], self.root / 'occt', self.root / 'runs')
        log.write_text('CASE bugs grid a: BAD (known problem)\n')
        source.write_text('changed\n')
        with self.assertRaisesRegex(ValueError, 'source'):
            audit.apply_runs([case], self.root / 'occt', self.root / 'runs')

    def test_native_run_rejects_unknown_cases_and_parity_claims(self):
        case, _, _ = self.execution_fixture()
        with self.assertRaisesRegex(ValueError, 'Unknown'):
            audit.apply_runs([], self.root / 'occt', self.root / 'runs')
        manifest = self.root / 'runs/first/results.json'
        data = json.loads(manifest.read_text())
        data['cases'][0]['comparison'] = 'equivalent'
        manifest.write_text(json.dumps(data))
        with self.assertRaisesRegex(ValueError, 'parity'):
            audit.apply_runs([case], self.root / 'occt', self.root / 'runs')

    def test_archived_logs_are_verified_without_extraction(self):
        case, _, log = self.execution_fixture()
        root = self.root / 'runs/first'
        archive = root / 'artifacts.zip'
        with zipfile.ZipFile(archive, 'w') as zipped:
            zipped.write(log, str(log.relative_to(root)))
        log.unlink()
        manifest = root / 'results.json'
        data = json.loads(manifest.read_text())
        data.update(artifact_archive=archive.name, artifact_archive_sha256=audit.sha(archive))
        manifest.write_text(json.dumps(data))
        audit.apply_runs([case], self.root / 'occt', self.root / 'runs')
        self.assertEqual(case['occt_run'], 'known-failure')
        archive.write_bytes(b'changed')
        with self.assertRaisesRegex(ValueError, 'artifact archive'):
            audit.apply_runs([case], self.root / 'occt', self.root / 'runs')


if __name__ == "__main__":
    unittest.main()
