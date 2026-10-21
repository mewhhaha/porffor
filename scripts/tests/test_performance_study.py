"""Attribution must consume admitted raw reports and retain real boundaries."""
from pathlib import Path
import json
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import performance_study
import test_benchmark_metrics as fixtures


class StudyTests(unittest.TestCase):
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
