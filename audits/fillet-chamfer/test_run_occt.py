"""Keep upstream harness outcomes distinct from process success and parity."""

import hashlib
from pathlib import Path
import tempfile
import unittest

import run_occt


class RunTests(unittest.TestCase):
    def test_known_failures_and_improvements_are_not_successes(self):
        for verdict, expected in [('OK', 'passed'), ('BAD', 'known-failure'),
                                  ('IMPROVEMENT', 'improvement'), ('FAILED', 'failed'),
                                  ('SKIPPED', 'skipped')]:
            result = run_occt.classify('bugs/grid/case', 0, False,
                                       f'CASE bugs grid case: {verdict} (detail)\n')
            self.assertEqual(result['status'], expected)
            self.assertEqual(result['harness_verdict'], verdict)

    def test_process_failure_and_timeout_override_a_printed_pass(self):
        log = 'CASE bugs grid case: OK\n'
        self.assertEqual(run_occt.classify('bugs/grid/case', -11, False, log)['status'], 'crash')
        self.assertEqual(run_occt.classify('bugs/grid/case', 0, True, log)['status'], 'timeout')
        self.assertEqual(run_occt.classify('bugs/grid/case', 2, False, log)['status'], 'harness-error')

    def test_exit_zero_without_the_correct_case_summary_is_not_a_pass(self):
        for log in ['', 'CASE bugs grid different: OK\n', 'fake CASE bugs grid case: OK\n']:
            self.assertEqual(run_occt.classify('bugs/grid/case', 0, False, log)['status'], 'harness-error')

    def test_tcl_words_cannot_inject_a_command(self):
        for value in ['ok', '/tmp/a path', '$name', 'a;exit', 'a[b]', 'a{b}', 'a\\b']:
            quoted = run_occt.tcl_word(value)
            self.assertTrue(quoted.startswith('"') and quoted.endswith('"'))
        self.assertEqual(run_occt.tcl_word('$x[a]'), '"\\$x\\[a\\]"')


class ExternalDataTests(unittest.TestCase):
    def test_external_data_preserves_default_search_path_and_records_exact_bytes(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            occt = root / 'occt'
            external = root / 'external shapes'
            nested = external / 'brep'
            nested.mkdir(parents=True)
            shape = nested / 'propag'
            content = b'raw shape\r\nbytes\x00'
            shape.write_bytes(content)
            data = run_occt.data_configuration(occt, [external])
            self.assertEqual(data['search_paths'], [str(occt / 'data'), str(external)])
            self.assertEqual(data['external_files'], {str(shape): hashlib.sha256(content).hexdigest()})
            self.assertEqual(shape.read_bytes(), content)
            shape.write_bytes(content + b'changed')
            self.assertNotEqual(data, run_occt.data_configuration(occt, [external]))
            self.assertEqual(run_occt.data_configuration(occt, [])['external_files'], {})

    def test_missing_external_data_directory_is_not_silently_ignored(self):
        with tempfile.TemporaryDirectory() as temp:
            with self.assertRaisesRegex(ValueError, 'directory'):
                run_occt.data_configuration(Path(temp), [Path(temp) / 'missing'])


if __name__ == '__main__':
    unittest.main()
