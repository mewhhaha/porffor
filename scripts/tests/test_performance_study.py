"""Attribution must consume admitted raw reports and retain real boundaries."""
from pathlib import Path
import json
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import performance_study
sys.path.insert(0, str(Path(__file__).resolve().parent))
import test_benchmark_metrics as fixtures


class StudyTests(unittest.TestCase):
    def test_nested_host_registration_is_retained_without_double_counting(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path, report = fixtures.EvidenceTests().make_report(root)
            case = report['execution']['wasmtime']['cases'][0]
            for observation in case['observations']:
                observation['phases_ns']['host_import_registration'] = 2
                observation['phases_ns']['admission'] = 7
            stdout = root / 'logs' / (case['command_label'] + '.stdout')
            stdout.write_text(json.dumps(dict(schema='scriptw-wasmtime-baseline-v2', samples=3, warmups=1,
                cases=[{key: value for key, value in case.items() if key != 'command_label'}])))
            for command in report['commands']:
                if command['label'] == case['command_label']:
                    command['stdout_sha256'] = performance_study.digest(stdout)
            for member in report['retained_files']:
                member['sha256'] = performance_study.digest(root / member['path'])
            report['summaries'] = fixtures.metrics.summarize(report)
            path.write_text(json.dumps(report))
            result = performance_study.study([path])['reports'][0]['cases'][0]
            self.assertEqual(result['unassigned_total_ns'], 970)
            self.assertEqual(result['phases_ns']['host_import_registration'], 2)
            self.assertEqual(result['nested_phases'], {'host_import_registration': 'store_and_linker'})

    def test_attributes_raw_phases_and_binds_source_report(self):
        with tempfile.TemporaryDirectory() as directory:
            path, report = fixtures.EvidenceTests().make_report(Path(directory))
            result = performance_study.study([path])
            case = result['reports'][0]['cases'][0]
            self.assertEqual(case['execution_ns'], 12)
            self.assertEqual(case['unassigned_total_ns'], 977)
            self.assertEqual(case['execution_fraction_of_total'], .012)
            self.assertEqual(case['source_compilation_profile'], 'unrecorded')
            self.assertEqual(result['row_scaling'], [])
            self.assertEqual(result['reports'][0]['sha256'], performance_study.digest(path))

    def test_does_not_attribute_tampered_or_incomplete_reports(self):
        with tempfile.TemporaryDirectory() as directory:
            path, report = fixtures.EvidenceTests().make_report(Path(directory))
            report['complete'] = False
            path.write_text(json.dumps(report))
            with self.assertRaises(ValueError):
                performance_study.study([path])
