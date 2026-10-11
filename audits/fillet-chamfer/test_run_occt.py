"""Keep upstream harness outcomes distinct from process success and parity."""

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


if __name__ == '__main__':
    unittest.main()
