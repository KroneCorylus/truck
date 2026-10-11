"""The capture wrapper must preserve native Tcl scope, results and errors."""

from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest


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


if __name__ == '__main__':
    unittest.main()
