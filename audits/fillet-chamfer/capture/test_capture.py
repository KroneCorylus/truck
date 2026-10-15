"""The capture wrapper must preserve native Tcl scope, results and errors."""

from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

from run_capture import selected_cases


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


if __name__ == '__main__':
    unittest.main()
