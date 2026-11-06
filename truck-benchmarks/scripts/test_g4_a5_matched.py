"""Evidence-integrity regressions; no kernel operations or timing measurements."""
import importlib.util
import json
import math
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch


path = Path(os.environ.get("NOV3_SCRIPT_UNDER_TEST", Path(__file__).with_name("g4_a5_matched.py")))
spec = importlib.util.spec_from_file_location("nov3_matched", path)
script = importlib.util.module_from_spec(spec)
spec.loader.exec_module(script)


class CampaignIntegrity(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.out = Path(self.temp.name)
        self.folder = self.out / "case"
        self.folder.mkdir()
        self.records = [{"case": script.CASES[0], "folder": str(self.folder), "hashes": {}}]
        script.save(self.out / "inputs.json", self.records)
        (self.out / "capture_helpers.rs").write_text("adapter")
        script.save(self.out / "build-provenance.json", {
            "source_stable": True, "after": {}, "helpers": script.digest(self.out / "capture_helpers.rs"),
            "binaries": {}, "libraries": {},
        })


    def test_da_preparation_binds_annotation_and_preserves_defaults(self):
        key = 'chamfer/dist_angle_complex/A5'
        self.assertIn(key, script.CASES)
        records = script.prepare(self.out / 'prepared')
        self.assertEqual(len(records), 11)
        annotations = {
            'chamfer/dist_dist_complex/A5': script.ROOT/'audits/fillet-chamfer/oracles/a5_dd_preference.json',
            key: script.ROOT/'audits/fillet-chamfer/oracles/a5_da_preference.json',
            'chamfer/equal_dist_complex/A6': script.ROOT/'target/a6-preference-kernel-20261104/annotated/A6/001.preferences.json',
        }
        for record in records:
            folder = Path(record['folder'])
            source = script.BASE/record['case']/'capture'
            for suffix in ('input.json', 'input.step', 'input.brep', 'result.json', 'result.brep'):
                name = '001.' + suffix
                self.assertEqual((folder/name).read_bytes(), (source/name).read_bytes())
                self.assertEqual(record['hashes'][name], script.digest(source/name))
            annotation = folder/'001.preferences.json'
            if record['case'] in annotations:
                self.assertEqual(annotation.read_bytes(), annotations[record['case']].read_bytes())
                self.assertEqual(record['hashes'][annotation.name], script.digest(annotation))
            else:
                self.assertFalse(annotation.exists())
            if record['case'] != key:
                continue
            bound = json.loads(annotation.read_text())
            self.assertEqual(len(bound['preferences']), 4)
            for preference in bound['preferences']:
                self.assertEqual(set(preference), {'vertex', 'edge'})
                self.assertEqual(len(preference['vertex']), 3)
                self.assertEqual(set(preference['edge']), {'points'})
                self.assertEqual(len(preference['edge']['points']), 2)
            for kind in ('step', 'json'):
                source_input = folder/('001.input.' + kind)
                self.assertEqual(bound['input_' + kind], source_input.read_text())
                self.assertEqual(bound['input_' + kind + '_sha256'], script.digest(source_input))
            selections = json.loads((folder/'001.input.json').read_text())['selection']
            lines = (folder/'native.config').read_text().splitlines()
            self.assertEqual(int(lines[0]), len(selections))
            for line, selection in zip(lines[1:], selections):
                plane = selection['reference_plane']
                expected = [3, selection['distance'], math.radians(selection['angle_degrees']),
                            *plane['origin'], *plane['normal'], len(selection['points']),
                            *(v for point in selection['points'] for v in point)]
                self.assertEqual(list(map(float, line.split())), expected)

    def material_approval(self, identity):
        proof, results = {}, []
        for record in self.records:
            folder = self.out/'smoke'/record['case'].replace('/', '-')
            folder.mkdir(parents=True, exist_ok=True)
            for engine in ('truck', 'occt'):
                step = folder/f'smoke-0-{engine}.step'
                step.write_text('validated geometry')
                proof[str(step.resolve())] = script.digest(step)
                results.append(dict(case=record['case'], engine=engine, points=3, mismatches=[]))
        approval = dict(identity=identity, proof=proof, results=results)
        script.save(self.out/'material-approved.json', approval)
        return approval

    def test_timing_requires_material_approval(self):
        with patch.object(script, 'campaign_identity', return_value={}), \
                patch.object(script, 'run', side_effect=self.successful_fake_run) as run:
            with self.assertRaises(FileNotFoundError):
                script.execute(self.out, self.records, False, 1, 1, None)
            run.assert_not_called()
        self.assertFalse((self.out/'time').exists())

    def test_invalid_material_approval_blocks_before_timing(self):
        for failure in ('stale', 'failed', 'missing_engine', 'duplicate', 'empty',
                        'missing_step', 'tampered_step', 'tampered_report'):
            with self.subTest(failure=failure):
                if (self.out/'time').exists():
                    (self.out/'time').rename(self.out/f'unexpected-attempt-before-{failure}')
                approval = self.material_approval({})
                report = self.out/'material-results.json'
                report.write_text('independent material report')
                approval['proof'][str(report)] = script.digest(report)
                if failure == 'stale': approval['identity'] = {'old_build': True}
                elif failure == 'failed': approval['results'][0]['mismatches'] = ['wrong material']
                elif failure == 'missing_engine': approval['results'].pop()
                elif failure == 'duplicate': approval['results'].append(approval['results'][0])
                elif failure == 'empty': approval['results'][0]['points'] = 0
                elif failure == 'missing_step': approval['proof'].pop(next(iter(approval['proof'])))
                elif failure == 'tampered_step': Path(next(iter(approval['proof']))).write_text('changed')
                elif failure == 'tampered_report': report.write_text('changed')
                script.save(self.out/'material-approved.json', approval)
                with patch.object(script, 'campaign_identity', return_value={}), \
                        patch.object(script, 'run', side_effect=self.successful_fake_run) as run:
                    with self.assertRaisesRegex(AssertionError, '[Mm]aterial'):
                        script.execute(self.out, self.records, False, 1, 1, None)
                    run.assert_not_called()
                self.assertFalse((self.out/'time').exists())

    def test_material_changes_during_timing_prevent_publication(self):
        for failure in ('approval', 'step', 'report'):
            with self.subTest(failure=failure):
                if (self.out/'time').exists():
                    (self.out/'time').rename(self.out/f'unexpected-attempt-before-{failure}')
                approval = self.material_approval({})
                report = self.out/'material-results.json'
                report.write_text('independent material report')
                approval['proof'][str(report)] = script.digest(report)
                script.save(self.out/'material-approved.json', approval)

                def fake_run(command, log, **kwargs):
                    self.successful_fake_run(command, log, **kwargs)
                    if failure == 'approval':
                        approval['results'][0]['points'] += 1
                        script.save(self.out/'material-approved.json', approval)
                    elif failure == 'step':
                        Path(next(iter(approval['proof']))).write_text('changed')
                    else:
                        report.write_text('changed')

                with patch.object(script, 'campaign_identity', return_value={}), \
                        patch.object(script, 'run', side_effect=fake_run):
                    with self.assertRaisesRegex(AssertionError, '[Mm]aterial'):
                        script.execute(self.out, self.records, False, 1, 1, None)
                self.assertFalse((self.out/'time/summary.json').exists())
                # Keep each failed attempt rather than reusing its stage.
                (self.out/'time').rename(self.out/f'failed-{failure}')

    def test_timing_refuses_smoke_from_another_build(self):
        cases = [dict(self.records[0], occt_identity={"passed": True})]
        # Both layouts make the same case approval available to old and new scripts.
        script.save(self.out / "smoke-results.json", cases)
        (self.out / "smoke").mkdir()
        script.save(self.out / "smoke/approval.json", {"identity": {"old_build": True}, "cases": cases})
        with patch.object(script, "sources", return_value={}), patch.object(script, "execute") as execute:
            with patch("sys.argv", [str(path), "time", "--output", str(self.out),
                                    "--serial-slot-reserved", "--cpu", "0"]):
                with self.assertRaisesRegex(AssertionError, "Smoke approval"):
                    script.main()
            execute.assert_not_called()

    def test_existing_timing_attempt_is_never_overwritten(self):
        (self.out / "time").mkdir()
        for location in [self.out / "summary.json", self.out / "time/summary.json"]:
            location.write_text("retained result")
        with patch.object(script, "campaign_identity", return_value={}, create=True):
            with patch.object(script, "run", side_effect=self.successful_fake_run) as run:
                with self.assertRaises(FileExistsError):
                    script.execute(self.out, self.records, False, 1, 1, None)
                run.assert_not_called()
        self.assertEqual((self.out / "time/summary.json").read_text(), "retained result")
        self.assertEqual((self.out / "summary.json").read_text(), "retained result")

    @staticmethod
    def successful_fake_run(command, log, **_):
        engine = "truck" if Path(command[0]).name in ("nov3_matched", "a6_matched", "g4_a5_matched") else "occt"
        log.write_text("\n".join(json.dumps({"engine": engine, "warmup": i == 0, "ms": 1.0})
                                 for i in range(2)) + "\n")

    def test_identity_drift_prevents_summary_publication(self):
        self.material_approval({'id': 1})
        with patch.object(script, "campaign_identity", side_effect=[{"id": 1}, {"id": 2}], create=True):
            with patch.object(script, "run", side_effect=self.successful_fake_run):
                with self.assertRaisesRegex(AssertionError, "identity"):
                    script.execute(self.out, self.records, False, 1, 1, None)
        self.assertFalse((self.out / "summary.json").exists())
        self.assertFalse((self.out / "time/summary.json").exists())

    def test_changed_input_config_invalidates_identity(self):
        config = self.folder / "native.config"
        config.write_text("old config")
        self.records[0]["hashes"] = {config.name: script.digest(config)}
        script.save(self.out / "inputs.json", self.records)
        with patch.object(script, "sources", return_value={}):
            original = script.campaign_identity(self.out)
            config.write_text("new config")
            with self.assertRaisesRegex(AssertionError, "Input/config"):
                script.campaign_identity(self.out)
            self.records[0]["hashes"][config.name] = script.digest(config)
            script.save(self.out / "inputs.json", self.records)
            self.assertNotEqual(original, script.campaign_identity(self.out))

    def test_optional_input_presence_is_bound(self):
        preference = self.folder / '001.preferences.json'
        with patch.object(script, 'sources', return_value={}):
            script.campaign_identity(self.out)
            preference.write_text('{}')
            with self.assertRaisesRegex(AssertionError, 'Input inventory'):
                script.campaign_identity(self.out)
            self.records[0]['hashes'][preference.name] = script.digest(preference)
            script.save(self.out/'inputs.json', self.records)
            script.campaign_identity(self.out)
            preference.unlink()
            with self.assertRaisesRegex(AssertionError, 'Input inventory'):
                script.campaign_identity(self.out)

    def test_added_preference_blocks_before_timing(self):
        with patch.object(script, 'sources', return_value={}):
            identity = script.campaign_identity(self.out)
            self.material_approval(identity)
            (self.folder/'001.preferences.json').write_text('{}')
            with patch.object(script, 'run', side_effect=self.successful_fake_run) as run:
                with self.assertRaisesRegex(AssertionError, 'Input inventory'):
                    script.execute(self.out, self.records, False, 1, 1, None)
                run.assert_not_called()
        self.assertFalse((self.out/'time').exists())

    def test_preference_added_during_execution_blocks_publication(self):
        def fake_run(command, log, **kwargs):
            self.successful_fake_run(command, log, **kwargs)
            (self.folder/'001.preferences.json').write_text('{}')
        with patch.object(script, 'sources', return_value={}):
            self.material_approval(script.campaign_identity(self.out))
            with patch.object(script, 'run', side_effect=fake_run):
                with self.assertRaisesRegex(AssertionError, 'Input inventory'):
                    script.execute(self.out, self.records, False, 1, 1, None)
        self.assertTrue((self.out/'time/records.jsonl').exists())
        self.assertFalse((self.out/'time/summary.json').exists())

    def test_timing_requires_both_engine_comparisons(self):
        (self.out / "smoke").mkdir()
        identity = {"validated_build": True}
        for engine in ("truck", "occt"):
            for passed in (False, None):
                with self.subTest(engine=engine, passed=passed):
                    proof = dict(self.records[0], truck_identity={"passed": True},
                                 occt_identity={"passed": True})
                    if passed is None:
                        del proof[engine + "_identity"]
                    else:
                        proof[engine + "_identity"]["passed"] = passed
                    script.save(self.out / "smoke/approval.json", {"identity": identity, "cases": [proof]})
                    with patch.object(script, "campaign_identity", return_value=identity), \
                            patch.object(script, "execute") as execute, \
                            patch("sys.argv", [str(path), "time", "--output", str(self.out),
                                               "--serial-slot-reserved", "--cpu", "0"]):
                        with self.assertRaisesRegex(AssertionError, "timing blocked"):
                            script.main()
                        execute.assert_not_called()

    def test_failed_build_stage_cannot_be_retried_in_place(self):
        (self.out / "build-stage").mkdir()
        (self.out / "build-provenance.json").unlink()
        (self.out / "capture_helpers.rs").unlink()
        with patch.object(script, "run") as run:
            with self.assertRaises(FileExistsError):
                script.build(self.out)
            run.assert_not_called()

    def test_matching_smoke_identity_allows_one_timing_attempt(self):
        def fake_run(command, log, **_):
            if command[0] == "taskset":
                command = command[3:]
            if Path(command[0]).name == "compare_shapes":
                script.save(Path(command[3]), {
                    "native_to_truck_max_distance": 0., "truck_to_native_max_distance": 0.,
                    "native_valid": True, "truck_step_valid": True, "native_solids": 1, "truck_solids": 1,
                })
                log.write_text("comparison\n")
                return
            engine = "truck" if Path(command[0]).name in ("nov3_matched", "a6_matched", "g4_a5_matched") else "occt"
            mode = command[2 if engine == "truck" else 3]
            count = 3 if mode == "smoke" else int(mode) + 1
            log.write_text("\n".join(json.dumps({"engine": engine, "warmup": i == 0,
                                                 **({} if mode == "smoke" else {"ms": 1.0})})
                                     for i in range(count)) + "\n")

        with patch.object(script, "sources", return_value={}), patch.object(script, "run", side_effect=fake_run):
            for action, extra in [("smoke", []), ("time", ["--serial-slot-reserved", "--cpu", "0",
                                                           "--samples", "2", "--rounds", "1"])]:
                if action == 'time':
                    self.material_approval(script.campaign_identity(self.out))
                with patch("sys.argv", [str(path), action, "--output", str(self.out), *extra]):
                    script.main()
        summary = json.loads((self.out / "time/summary.json").read_text())
        self.assertEqual([row["samples"] for row in summary], [2, 2])


if __name__ == "__main__":
    unittest.main()
