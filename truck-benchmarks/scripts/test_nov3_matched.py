"""Evidence-integrity regressions; no kernel operations or timing measurements."""
import importlib.util
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch


path = Path(os.environ.get("NOV3_SCRIPT_UNDER_TEST", Path(__file__).with_name("nov3_matched.py")))
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
        engine = "truck" if Path(command[0]).name in ("nov3_matched", "a6_matched") else "occt"
        log.write_text("\n".join(json.dumps({"engine": engine, "warmup": i == 0, "ms": 1.0})
                                 for i in range(2)) + "\n")

    def test_identity_drift_prevents_summary_publication(self):
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
            engine = "truck" if Path(command[0]).name in ("nov3_matched", "a6_matched") else "occt"
            mode = command[2 if engine == "truck" else 3]
            count = 3 if mode == "smoke" else int(mode) + 1
            log.write_text("\n".join(json.dumps({"engine": engine, "warmup": i == 0,
                                                 **({} if mode == "smoke" else {"ms": 1.0})})
                                     for i in range(count)) + "\n")

        with patch.object(script, "sources", return_value={}), patch.object(script, "run", side_effect=fake_run):
            for action, extra in [("smoke", []), ("time", ["--serial-slot-reserved", "--cpu", "0",
                                                           "--samples", "2", "--rounds", "1"])]:
                with patch("sys.argv", [str(path), action, "--output", str(self.out), *extra]):
                    script.main()
        summary = json.loads((self.out / "time/summary.json").read_text())
        self.assertEqual([row["samples"] for row in summary], [2, 2])


if __name__ == "__main__":
    unittest.main()
