"""Observed spread must retain raw samples and reject mixed or damaged evidence."""
import hashlib
import json
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import paired_repeatability as analysis


def fingerprint(value):
    return hashlib.sha256(value.encode()).hexdigest()


def fixture(root):
    identities = {side: dict(source_revision=dict(commit=side),
                            source_fingerprint=fingerprint(side)) for side in ('before', 'after')}
    runners = {side: dict(sha256=fingerprint(side + '-runner')) for side in identities}
    receipt = dict(schema='scriptw-frozen-string-hash-diagnostic-v1', complete=True,
                   configuration=dict(samples=3, warmups=1), cpu=[0],
                   harness_sha256=fingerprint('harness'), identities=identities,
                   runners=runners, observations=[])
    for index, (side, value) in enumerate([('before', 100), ('after', 80),
                                          ('after', 88), ('before', 110)]):
        raw_name = f'case.js-{index}-{side}.json'
        case = dict(file='case.js', source_sha256=fingerprint('fixture'),
                    wasm_sha256=fingerprint(side + '-wasm'), linked_runtime=dict(key=side),
                    observations=[dict(warmup=warmup, execution_ns=value, stdout='7\n',
                                       runtime_policy='policy', module_cache='Hit')
                                  for warmup in [True, False, False, False]])
        raw = dict(schema='scriptw-wasmtime-baseline-v2', samples=3, warmups=1,
                   source_commit=side, source_fingerprint=identities[side]['source_fingerprint'],
                   executable_sha256=runners[side]['sha256'], cases=[case])
        raw_path = root / raw_name
        raw_path.write_text(json.dumps(raw))
        receipt['observations'].append(dict(
            file=case['file'], side=side, label=raw_name[:-5], raw=raw_name,
            raw_sha256=analysis.sha(raw_path), execution_ns=[value] * 3, median_ns=value,
            process=dict(exit_code=0, timed_out=False), source_commit=side,
            source_fingerprint=raw['source_fingerprint'], runner_sha256=raw['executable_sha256'],
            source_sha256=case['source_sha256'], wasm_sha256=case['wasm_sha256']))
    path = root / 'receipt.json'
    path.write_text(json.dumps(receipt))
    return path, receipt


class RepeatabilityTests(unittest.TestCase):
    def test_retains_rounds_and_computes_source_spread_and_adjacent_effects(self):
        with tempfile.TemporaryDirectory() as directory:
            path, _ = fixture(Path(directory))
            result = analysis.analyze(path)
            case, = result['cases']
            self.assertEqual(case['cohorts']['before']['round_medians_ns'], [100, 110])
            self.assertEqual(case['cohorts']['after']['sample_range_ns'], [80, 88])
            self.assertAlmostEqual(case['cohorts']['before']['round_median_spread_percent'], 10)
            for effect in case['paired_effect_percent'] + [case['median_ratio_effect_percent']]:
                self.assertAlmostEqual(effect, -20)
            self.assertEqual(result['receipt_sha256'], analysis.sha(path))

    def test_rejects_tampered_raw_bytes_without_rewriting_the_receipt(self):
        with tempfile.TemporaryDirectory() as directory:
            path, receipt = fixture(Path(directory))
            raw = path.parent / receipt['observations'][0]['raw']
            raw.write_text(raw.read_text() + '\n')
            with self.assertRaisesRegex(ValueError, 'checksum'):
                analysis.analyze(path)

    def test_rejects_mixed_or_incomplete_rounds(self):
        for change, error in [('incomplete', 'incomplete'), ('order', 'ABBA'),
                              ('source', 'source revision'), ('median', 'median'),
                              ('zero', 'elapsed samples')]:
            with self.subTest(change=change), tempfile.TemporaryDirectory() as directory:
                path, receipt = fixture(Path(directory))
                row = receipt['observations'][0]
                if change == 'incomplete':
                    receipt['observations'].pop()
                elif change == 'order':
                    row['side'] = 'after'
                elif change == 'source':
                    row['source_commit'] = 'another-build'
                elif change == 'median':
                    row['median_ns'] = 99
                else:
                    row['execution_ns'][0] = 0
                path.write_text(json.dumps(receipt))
                with self.assertRaisesRegex(ValueError, error):
                    analysis.analyze(path)

    def test_rejects_changed_stdout_and_cache_even_with_updated_raw_digest(self):
        for change, error in [('stdout', 'stdout'), ('cache', 'module cache'),
                              ('runtime', 'same-source'), ('warmups', 'warmup ordering')]:
            with self.subTest(change=change), tempfile.TemporaryDirectory() as directory:
                path, receipt = fixture(Path(directory))
                row = receipt['observations'][3]
                raw_path = path.parent / row['raw']
                raw = json.loads(raw_path.read_text())
                case = raw['cases'][0]
                if change == 'stdout':
                    case['observations'][1]['stdout'] = '8\n'
                elif change == 'cache':
                    case['observations'][1]['module_cache'] = 'Miss'
                elif change == 'runtime':
                    case['linked_runtime']['key'] = 'another-runtime'
                else:
                    case['observations'][0]['warmup'] = False
                raw_path.write_text(json.dumps(raw))
                row['raw_sha256'] = analysis.sha(raw_path)
                path.write_text(json.dumps(receipt))
                with self.assertRaisesRegex(ValueError, error):
                    analysis.analyze(path)


if __name__ == '__main__':
    unittest.main()
