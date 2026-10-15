"""The capture wrapper must preserve native Tcl scope, results and errors."""

import json
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch
import shutil
import subprocess
import tempfile
import unittest

from run_capture import run_case, selected_cases


class WrapperTests(unittest.TestCase):
    def run_wrapper(self, body):
        tcl = shutil.which('tclsh')
        if not tcl:
            self.skipTest('Tcl is not installed')
        wrapper = Path(__file__).with_name('blend_capture.tcl')
        with tempfile.TemporaryDirectory() as temp:
            script = Path(temp) / 'test.tcl'
            script.write_text('proc audit_blend_input {args} {return captured}\n'
                              'proc audit_blend_output {args} {return captured}\n' + body.replace(
                                  '# WRAPPER', f'source {{{wrapper}}}'))
            result = subprocess.run([tcl, str(script)], cwd=temp, capture_output=True,
                                    text=True, timeout=10)
            self.assertEqual(result.returncode, 0, result.stderr)
            return result.stdout

    def test_native_and_capture_commands_use_the_callers_scope(self):
        output = self.run_wrapper('''
proc blend {args} {upvar 1 shape value; set value changed; return native-result}
proc audit_blend_input {args} {upvar 1 shape value; if {$value ne "original"} {error scope}}
proc audit_blend_output {args} {upvar 1 shape value; if {$value ne "changed"} {error scope}}
# WRAPPER
proc scenario {} {
    set shape original
    set result [blend result shape 1 edge]
    puts [list $result $shape]
}
scenario
set log [open capture/001.capture.log]
puts [read $log]
close $log
''')
        self.assertIn('native-result changed', output)
        self.assertIn('input_captured 1', output)
        self.assertIn('output_captured 1', output)

    def test_capture_error_does_not_change_native_success_or_error_options(self):
        output = self.run_wrapper('''
proc blend {args} {
    if {[lindex $args 0] eq "bad"} {return -code error -errorcode {NATIVE CUSTOM} native-error}
    return native-success
}
proc audit_blend_input {args} {error capture-error}
# WRAPPER
puts [blend good shape 1 edge]
set code [catch {blend bad shape 1 edge} message options]
puts [list $code $message [dict get $options -errorcode]]
''')
        self.assertEqual(output.strip().splitlines(),
                         ['native-success', '1 native-error {NATIVE CUSTOM}'])

    def test_chamfer_capture_preserves_scope_and_interleaves_with_fillets(self):
        output = self.run_wrapper('''
proc blend {args} {return fillet-result}
proc chamf {args} {upvar 1 shape value; set value changed; return chamfer-result}
proc audit_chamfer_input {args} {upvar 1 shape value; if {$value ne "original"} {error scope}}
proc audit_blend_output {args} {upvar 1 shape value; if {$value ne "changed"} {error scope}}
# WRAPPER
proc scenario {} {
    set shape original
    puts [chamf result shape edge 2]
    puts $shape
    puts [blend result shape 1 edge]
}
scenario
set log [open capture/001.capture.log]
puts [read $log]
close $log
puts [file exists capture/002.capture.log]
''')
        self.assertIn('chamfer-result\nchanged\nfillet-result', output)
        self.assertIn('input_captured 1', output)
        self.assertIn('output_captured 1', output)
        self.assertTrue(output.rstrip().endswith('1'))

    def test_chamfer_capture_failure_preserves_native_error_options(self):
        output = self.run_wrapper('''
proc blend {args} {return unused}
proc chamf {args} {return -code error -errorcode {CHAMFER CUSTOM} native-error}
proc audit_chamfer_input {args} {error capture-error}
# WRAPPER
set code [catch {chamf result shape edge 2} message options]
puts [list $code $message [dict get $options -errorcode]]
puts [file exists capture/001.capture.log]
''')
        self.assertEqual(output.strip().splitlines(), ['1 native-error {CHAMFER CUSTOM}', '1'])


class SelectionTests(unittest.TestCase):
    def test_native_chamfer_tokens_are_selected_without_relabeling_missing_fixtures(self):
        cases = [dict(id='bugs/a/blend', occt_run='passed', commands=['blend']),
                 dict(id='bugs/a/chamfer', occt_run='passed', commands=['chamf']),
                 dict(id='chamfer/equal_dist/A1', occt_run='skipped', commands=[]),
                 dict(id='bugs/a/unrelated', occt_run='passed', commands=[])]
        self.assertEqual(selected_cases(dict(cases=cases), []), cases[:2])
        self.assertEqual(selected_cases(dict(cases=cases), ['*chamfer']), [cases[1]])

    def test_passing_dedicated_chamfers_are_selected_when_helpers_build_the_command(self):
        passing = dict(id='chamfer/equal_dist/A1', occt_run='passed', commands=[])
        cases = [passing, dict(id='chamfer/equal_dist/A2', occt_run='skipped', commands=[]),
                 dict(id='chamfer/dist_angle/A3', occt_run='known-failure', commands=[]),
                 dict(id='bugs/grid/other', occt_run='passed', commands=[])]
        self.assertEqual(selected_cases(dict(cases=cases), []), [passing])
        self.assertEqual(selected_cases(dict(cases=cases), ['chamfer/*']), [passing])


class RejectedCaptureTests(unittest.TestCase):
    def capture(self, log, *, input_json=True, input_step=True, result_json=None,
                truck_status='operation-failed', task=None):
        with tempfile.TemporaryDirectory() as temp:
            args = SimpleNamespace(output=Path(temp), draw='DRAWEXE', replay=Path('replay'),
                                   plugin=Path('plugin'), timeout=10)
            if task is None:
                task = dict(operation='fillet', selection=[dict(radius=50, points=[[0, 0, 0], [0, 0, 100]])])
            calls = []

            def execute(command, directory, env, timeout):
                calls.append(command)
                prefix = directory / 'capture' / '001'
                if command[0] == 'DRAWEXE':
                    prefix.parent.mkdir()
                    prefix.with_suffix('.input.json').write_text(json.dumps(task) if input_json else '{')
                    if input_step:
                        prefix.with_suffix('.input.step').write_text('input fixture')
                    if log is not None:
                        prefix.with_suffix('.capture.log').write_text(log)
                    if result_json is not None:
                        prefix.with_suffix('.result.json').write_text(result_json)
                elif command[0] == 'replay':
                    report = dict(comparison='unverified', result=dict(status=truck_status,
                                  input_unchanged=True, step_prepared=truck_status=='operation-succeeded'))
                    prefix.with_suffix('.truck.json').write_text(json.dumps(report))
                else:
                    self.fail('A rejected native operation has no reference shape to compare')
                return '', 0, False, 0.

            with patch('run_capture.execute', side_effect=execute), patch('run_capture.classify', return_value=dict(status='passed')):
                result = run_case(dict(id='bugs/grid/case', occt_run='passed'), {}, args, {})
            return result['captures'][0], calls

    def test_confirmed_native_rejection_replays_input_without_shape_comparison(self):
        capture, calls = self.capture('input_captured 1 message {}\nnative_return_code 1\n')
        self.assertEqual(len(calls), 2)
        self.assertEqual(capture['native_result'], dict(status='operation-failed', returncode=1))
        self.assertEqual(capture['truck']['result']['status'], 'operation-failed')
        self.assertTrue(capture['truck']['result']['input_unchanged'])
        self.assertEqual(capture['task']['selection'], [dict(radius=50, point_count=2)])
        self.assertIsNone(capture['sampled_comparison'])
        self.assertEqual(capture['comparison'], 'unverified')

    def test_truck_success_against_native_rejection_does_not_invent_a_comparison(self):
        capture, calls = self.capture('input_captured 1 message {}\nnative_return_code 1\n', truck_status='operation-succeeded')
        self.assertEqual(len(calls), 2)
        self.assertEqual(capture['truck']['result']['status'], 'operation-succeeded')
        self.assertEqual(capture['native_result']['status'], 'operation-failed')
        self.assertIsNone(capture['sampled_comparison'])
        self.assertEqual(capture['comparison'], 'unverified')

    def test_missing_or_corrupt_capture_evidence_remains_incomplete(self):
        confirmed = 'input_captured 1 message {}\nnative_return_code 1\n'
        scenarios = [dict(log=None), dict(log=''),
                     dict(log='input_captured 1 message {}\nnative_return_code 0\n'),
                     dict(log='input_captured 0 message export-error\nnative_return_code 1\n'),
                     dict(log='input_captured 1 message {}\nnative_return_code 2\n'),
                     dict(log=confirmed, input_json=False), dict(log=confirmed, input_step=False),
                     dict(log=confirmed, result_json='{')]
        for scenario in scenarios:
            with self.subTest(scenario=scenario):
                capture, calls = self.capture(**scenario)
                self.assertEqual(len(calls), 1)
                self.assertEqual(capture['status'], 'capture-incomplete')
                self.assertNotIn('truck', capture)

    def test_native_contour_summary_preserves_scope_without_repeating_point_arrays(self):
        edge = dict(curve_type=0, points=[[0, 0, 0], [1, 0, 0]])
        chosen = dict(mode='equal-distance', distance=1, native_applied=True,
                      ignored_reason=None, contour_edges=[edge, edge], **edge)
        ignored = dict(mode='equal-distance', distance=-1, native_applied=False,
                       ignored_reason='already-selected', contour_edges=[], **edge)
        task = dict(schema_version=2, operation='chamfer', selection=[chosen, ignored])
        capture, _ = self.capture('input_captured 1 message {}\nnative_return_code 1\n', task=task)
        selected = capture['task']['selection']
        self.assertEqual(capture['task']['schema_version'], 2)
        self.assertEqual(selected[0]['contour_edges'], [dict(curve_type=0, point_count=2)] * 2)
        self.assertTrue(selected[0]['native_applied'])
        self.assertIsNone(selected[0]['ignored_reason'])
        self.assertFalse(selected[1]['native_applied'])
        self.assertEqual(selected[1]['ignored_reason'], 'already-selected')
        self.assertEqual(selected[1]['contour_edges'], [])
        self.assertEqual(selected[0]['point_count'], 2)
        self.assertNotIn('points', selected[0])


if __name__ == '__main__':
    unittest.main()
