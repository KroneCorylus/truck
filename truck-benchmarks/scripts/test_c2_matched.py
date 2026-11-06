"""C2 preparation and approval integrity only; never invokes a kernel."""
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import c2_matched as c2
import test_g4_a5_matched as shared_tests


class C2Integrity(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.out = Path(self.temp.name)
        self.h = c2.harness
        self.record = dict(case=self.h.CASES[0], folder=str(self.out/'input'), hashes={})
        row = dict(self.record, truck_identity={'passed': True}, occt_identity={'passed': True})
        self.smoke = dict(identity={'build': 1}, cases=[row])
        folder = self.out/'smoke'/self.record['case'].replace('/', '-')
        folder.mkdir(parents=True)
        for engine in ('truck', 'occt'):
            (folder/f'smoke-0-{engine}.step').write_text('synthetic integrity fixture')
        self.h.save(self.out/'smoke/approval.json', self.smoke)
        self.approval = dict(identity={'build': 1}, proof={}, results=[
            dict(case=self.record['case'], engine=e, points=3, mismatches=[])
            for e in ('truck', 'occt')])
        self.seal()

    def seal(self):
        self.approval['proof'] = {str(p.resolve()): self.h.digest(p)
                                  for p in (self.out/'smoke').rglob('*') if p.is_file()}
        self.h.save(self.out/'material-approved.json', self.approval)

    def check(self):
        return c2.material_identity(self.out/'material-approved.json', self.out,
                                    [self.record], {'build': 1})

    def test_prepare_c2_modes_and_retain_neighbors(self):
        with patch.object(self.h, 'run', side_effect=AssertionError('No processes')):
            records = self.h.prepare(self.out/'prepared')
        self.assertEqual(len(records), 14)
        for record, mode in zip(records[:3], (1, 2, 3)):
            folder = Path(record['folder'])
            for suffix in ('input.json', 'input.step', 'input.brep', 'result.json', 'result.brep'):
                self.assertEqual((folder/('001.'+suffix)).read_bytes(),
                                 (self.h.BASE/record['case']/'capture'/('001.'+suffix)).read_bytes())
            lines = (folder/'native.config').read_text().splitlines()
            self.assertEqual(int(lines[0]), len(lines)-1)
            self.assertTrue(all(int(line.split()[0]) == mode for line in lines[1:]))
        with self.assertRaises(FileExistsError):
            self.h.prepare(self.out/'prepared')

    def test_reject_stale_failed_missing_duplicate_smoke(self):
        for failure in ('stale', 'failed', 'missing', 'duplicate', 'input'):
            with self.subTest(failure=failure):
                smoke = json.loads(json.dumps(self.smoke))
                if failure == 'stale': smoke['identity'] = {}
                if failure == 'failed': smoke['cases'][0]['truck_blocker'] = 'failed'
                if failure == 'missing': del smoke['cases'][0]['occt_identity']
                if failure == 'duplicate': smoke['cases'] *= 2
                if failure == 'input': smoke['cases'][0]['hashes'] = {'changed': 'hash'}
                self.h.save(self.out/'smoke/approval.json', smoke)
                self.seal()
                with self.assertRaises(AssertionError): self.check()

    def test_smoke_evidence_must_be_bound_and_unchanged(self):
        self.check()
        log = self.out/'smoke/new.log'
        log.write_text('raw QA evidence')
        with self.assertRaises(AssertionError): self.check()
        self.seal()
        self.check()
        log.write_text('changed')
        with self.assertRaises(AssertionError): self.check()

    def test_smoke_drift_prevents_publication(self):
        def fake_run(command, log, **kwargs):
            shared_tests.CampaignIntegrity.successful_fake_run(command, log, **kwargs)
            (self.out/'smoke/approval.json').write_text('{}')
        with patch.object(self.h, 'campaign_identity', return_value={'build': 1}), \
                patch.object(self.h, 'run', side_effect=fake_run):
            with self.assertRaises(AssertionError):
                self.h.execute(self.out, [self.record], False, 1, 1, None)
        self.assertFalse((self.out/'time/summary.json').exists())
        self.assertTrue((self.out/'time').is_dir())

    def test_wrapper_and_tests_are_source_bound(self):
        sources = c2.sources()
        for name in ('c2_matched.py', 'test_c2_matched.py', 'g4_a5_matched.py'):
            self.assertIn('truck-benchmarks/scripts/'+name, sources)


if __name__ == '__main__':
    unittest.main()
