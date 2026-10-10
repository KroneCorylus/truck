"""Reporting must never turn an incomplete or invalid benchmark into a speedup."""
import copy
import json
import unittest

from compare_occt import parse_result, summarize


def qa():
    return dict(pass_=True, volume=10.0, expected_volume=10.0, relative_volume_error=0.0,
                geometric=True, mesh_closed=True, qa_tolerance=.001)


def output(samples):
    checks = qa()
    checks['pass'] = checks.pop('pass_')
    return [dict(phase='timing',samples_ms=samples,peak_rss_kib=1024),dict(phase='qa',qa=checks)]


def parse(records, count, status='complete'):
    return parse_result('\n'.join('RESULT '+json.dumps(r) for r in records),count,dict(status=status))


def measurements():
    records = []
    for engine in ('truck','occt'):
        for round_ in (0,1,2):
            samples = [] if round_==0 else ([1,2,3] if engine=='truck' else [3,4,5])
            row = parse(output(samples),len(samples))
            row.update(case='holes_batch/1',engine=engine,workers=1,round=round_)
            records.append(row)
    return records


class CorrectnessGates(unittest.TestCase):
    def test_success_uses_round_medians_and_keeps_tail(self):
        result = summarize(measurements(),['holes_batch/1'],[1],2)[0]
        self.assertEqual(result['ratio'],2)
        self.assertEqual(result['truck']['p95_ms'],3)

    def test_failed_preflight_or_timed_round_never_gets_ratio(self):
        for index in (0,1,2):
            for status in ('invalid','failed','timeout','protocol_error'):
                records = measurements()
                records[index]['status'] = status
                self.assertIsNone(summarize(records,['holes_batch/1'],[1],2)[0]['ratio'])

    def test_incomplete_or_duplicate_coverage_is_rejected(self):
        records = measurements()
        for broken in (records[:-1],records+[records[0]]):
            with self.assertRaises(ValueError):
                summarize(broken,['holes_batch/1'],[1],2)

    def test_fixture_mismatch_is_rejected(self):
        records = measurements()
        records[-1]['qa']['expected_volume'] = 11
        with self.assertRaises(ValueError):
            summarize(records,['holes_batch/1'],[1],2)

    def test_individually_acceptable_but_disagreeing_volumes_get_no_ratio(self):
        records = measurements()
        records[2]['qa']['volume'] = 9.985
        records[5]['qa']['volume'] = 10.015
        self.assertIsNone(summarize(records,['holes_batch/1'],[1],2)[0]['ratio'])

    def test_invalid_samples_and_missing_qa_are_rejected(self):
        for samples in ([1], [1,float('nan')], [1,-1], [1,0]):
            self.assertEqual(parse(output(samples),2)['status'],'protocol_error')
        self.assertEqual(parse(output([1])[:1],1)['status'],'protocol_error')
        records = output([1])
        self.assertEqual(parse(records+[copy.deepcopy(records[0])],1)['status'],'protocol_error')

    def test_failing_native_qa_and_nonfinite_volume_are_rejected(self):
        records = output([1])
        records[1]['qa']['pass'] = False
        self.assertEqual(parse(records,1)['status'],'invalid')
        records[1]['qa']['pass'] = True
        records[1]['qa']['volume'] = float('nan')
        self.assertEqual(parse(records,1)['status'],'protocol_error')

    def test_inconsistent_volume_error_is_rejected(self):
        records = output([1])
        records[1]['qa']['volume'] = 10.1
        self.assertEqual(parse(records,1)['status'],'protocol_error')

    def test_timeout_preserves_timings_without_claiming_success(self):
        result = parse(output([1])[:1],1,'timeout')
        self.assertEqual(result['status'],'timeout')
        self.assertEqual(result['timing']['samples_ms'],[1])


if __name__ == '__main__':
    unittest.main()
