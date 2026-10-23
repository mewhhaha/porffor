"""Reject invalid phase evidence before publishing timing or CPU attribution."""
import copy
import json
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from benchmark_metrics import digest
from measure_row_phases import check_map, validate


class PhaseEvidenceTests(unittest.TestCase):
    def fixture(self):
        entry = dict(file='rows.js', source_sha256='source', stdout='checked\n')
        rows = []
        for warmup in (True, False, False, False):
            rows.append(dict(warmup=warmup, stdout=entry['stdout'], execution_ns=1000,
                             module_cache='Miss' if warmup else 'Hit', runtime_policy='gc',
                             row_phases=dict(construction_ns=300, traversal_ns=400,
                                             combined_ns=710, middle_callback_ns=10,
                                             perf_region='traversal', perf_sampled=not warmup,
                                             perf_control_spans=[] if warmup else
                                             [dict(enable=True), dict(enable=False)])))
        return entry, dict(schema='scriptw-wasmtime-row-phases-v1', samples=3, warmups=1,
                           executable_sha256='runner', source_commit='source53', source_fingerprint='fp',
                           cases=[dict(**entry, observations=rows, wasm_sha256='program',
                                       linked_runtime=dict(wasm_sha256='runtime'))])

    def test_valid_profile_and_ordinary_observations(self):
        entry, raw = self.fixture()
        identity = validate(raw, entry, 'runner', 3, 1, True, 'traversal')
        self.assertEqual(identity['wasm_sha256'], 'program')
        raw['schema'] = 'scriptw-wasmtime-baseline-v2'
        for row in raw['cases'][0]['observations']:
            del row['row_phases']
        self.assertEqual(validate(raw, entry, 'runner', 3, 1, False, None), identity)

    def test_changed_execution_or_sampling_boundaries_are_rejected(self):
        entry, original = self.fixture()
        mutations = [
            lambda raw: raw.update(executable_sha256='other'),
            lambda raw: raw.update(samples=4),
            lambda raw: raw['cases'][0].update(source_sha256='other'),
            lambda raw: raw['cases'][0]['observations'][1].update(stdout='wrong'),
            lambda raw: raw['cases'][0]['observations'][1].update(module_cache='Miss'),
            lambda raw: raw['cases'][0]['observations'][1].update(runtime_policy='other'),
            lambda raw: raw['cases'][0]['observations'][1].update(warmup=True),
            lambda raw: raw['cases'][0]['observations'][1]['row_phases'].update(combined_ns=711),
            lambda raw: raw['cases'][0]['observations'][0]['row_phases'].update(perf_sampled=True),
            lambda raw: raw['cases'][0]['observations'][1]['row_phases'].update(perf_control_spans=[]),
        ]
        for index, mutate in enumerate(mutations):
            with self.subTest(index=index):
                raw = copy.deepcopy(original)
                mutate(raw)
                with self.assertRaises(ValueError):
                    validate(raw, entry, 'runner', 3, 1, True, 'traversal')

    def test_map_requires_process_artifact_and_byte_identity(self):
        with tempfile.TemporaryDirectory() as directory:
            mapping = Path(directory) / 'perf-123.map'
            path = Path(directory) / 'perf-123.map.identity'
            mapping.write_text('100 20 scriptw:R:f0:helper\n')
            source = ('scriptw-perf-map-v1 pid=123 boundary=serial-single-program-snapshot\n'
                      'R wasm_sha256=runtime native_text_sha256=' + 'a' * 64 + ' base=100 bytes=32\n'
                      'P wasm_sha256=program native_text_sha256=' + 'b' * 64 + ' base=200 bytes=16\n'
                      'map_sha256=' + digest(mapping) + '\n')
            case = dict(wasm_sha256='program', linked_runtime=dict(wasm_sha256='runtime'))
            path.write_text(source)
            self.assertEqual(check_map(path, case)['pid'], 123)
            for changed in (source.replace('pid=123', 'pid=124'),
                            source.replace('wasm_sha256=program', 'wasm_sha256=other'),
                            source.replace('wasm_sha256=runtime', 'wasm_sha256=other')):
                path.write_text(changed)
                with self.assertRaises(ValueError):
                    check_map(path, case)
            path.write_text(source)
            mapping.write_text('tampered\n')
            with self.assertRaises(ValueError):
                check_map(path, case)
