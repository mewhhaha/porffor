"""Admission of complete ordinary frozen-SDK retention observations."""
import copy
import json
from pathlib import Path
import sys
import unittest

SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))
import startup_retention_cohort as cohort

FIXTURES = SCRIPTS.parent / 'crates/scriptw-cli/tests/fixtures/v8_baseline'
MANIFEST = json.loads((FIXTURES / 'manifest.json').read_text())
CASE = 'objects.js'
EXPECTED = next(case['stdout'] for case in MANIFEST['execution'] if case['file'] == CASE)
SOURCE_SHA = cohort.digest(FIXTURES / CASE)
EXE_SHA = 'a' * 64
SOURCE_COMMIT = '30fc1931471646253e339100a6d392b2337a373b'
RUNTIME_POLICY = ('reference-types=required function-references=required gc=required '
                  'exceptions=required threads=required shared-memory=required collector=copying '
                  'cycle-collection=available js-semantic-heap=wasm-gc '
                  'weak-references=copying-collector ephemerons=owner-key-fixed-point')


def payload():
    # Same wire fields as the retained30fc baseline-v2 SDK JSON. The fixture's
    # checksum comes from the product manifest, not a second copied oracle.
    rows = []
    for index in range(5):
        rows.append(dict(warmup=index < 2, stdout=EXPECTED, execution_ns=10,
            module_cache='Miss' if index == 0 else 'Hit',
            host_import_cache='Miss' if index == 0 else 'Hit',
            matched_runtime_image_cache='NotAccessed', runtime_policy=RUNTIME_POLICY,
            phases_ns=dict(admission=1, engine=1, module=20, store_and_linker=1,
                           host_import_registration=1, instantiate=1, export_lookup=1, total=100),
            memory=dict(gc_capacity_before_bytes=16 * 1024 ** 2,
                        gc_capacity_after_bytes=16 * 1024 ** 2)))
    return dict(schema='scriptw-wasmtime-baseline-v2', samples=3, warmups=2,
        source_commit=SOURCE_COMMIT, source_fingerprint='b' * 64,
        executable_sha256=EXE_SHA, requested_native_policy=None,
        cases=[dict(file=CASE, source_sha256=SOURCE_SHA, source_compilation_profile='portable',
                    wasm_sha256='c' * 64, wasm_bytes=128,
                    linked_runtime=dict(key='d' * 64, wasm_sha256='e' * 64, wasm_bytes=256),
                    observations=rows)])


def validate(raw, identity=None):
    return cohort.validate_baseline_payload(raw,
        case_name=CASE, fixture_sha256=SOURCE_SHA, executable_sha256=EXE_SHA,
        source_commit=SOURCE_COMMIT, expected_stdout=EXPECTED, samples=3, warmups=2,
        identity=identity)


class StartupRetentionAdmissionTests(unittest.TestCase):
    def test_expected_stdout_is_bound_to_exactly_one_manifest_case(self):
        self.assertEqual(cohort.manifest_stdout(MANIFEST, CASE), EXPECTED)
        for mutation in ('schema', 'missing', 'duplicate', 'nonstring', 'inventory'):
            changed = copy.deepcopy(MANIFEST)
            entry = next(case for case in changed['execution'] if case['file'] == CASE)
            if mutation == 'schema': changed['schema'] = 'other'
            if mutation == 'missing': changed['execution'].remove(entry)
            if mutation == 'duplicate': changed['execution'].append(copy.deepcopy(entry))
            if mutation == 'nonstring': entry['stdout'] = 149600
            if mutation == 'inventory': changed['execution'] = None
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                cohort.manifest_stdout(changed, CASE)

    def test_complete_ordinary_sdk_rows_are_admitted_without_inventing_r_cache_observations(self):
        raw = payload()
        case, measured, identity = validate(raw)
        self.assertIs(case, raw['cases'][0])
        self.assertEqual(len(measured), 3)
        self.assertTrue(all(row['warmup'] is False and row['stdout'] == EXPECTED for row in measured))
        self.assertTrue(all(row['module_cache'] == 'Hit' for row in measured))
        self.assertTrue(all('runtime_module_cache' not in row and 'retained_cache_entries' not in row
                            for row in raw['cases'][0]['observations']))
        self.assertEqual(validate(payload(), identity)[2], identity)

    def test_missing_extra_or_reordered_warmups_and_checked_rows_are_rejected(self):
        for mutation in ('lost_warmup', 'lost_checked', 'extra_warmup', 'extra_checked',
                         'swapped', 'integer_warmup', 'missing_warmup', 'sample_plan',
                         'boolean_warmup_count', 'two_cases'):
            raw = payload()
            rows = raw['cases'][0]['observations']
            if mutation == 'lost_warmup': rows.pop(0)
            if mutation == 'lost_checked': rows.pop()
            if mutation == 'extra_warmup': rows.insert(0, copy.deepcopy(rows[0]))
            if mutation == 'extra_checked': rows.append(copy.deepcopy(rows[-1]))
            if mutation == 'swapped': rows[1], rows[2] = rows[2], rows[1]
            if mutation == 'integer_warmup': rows[0]['warmup'] = 1
            if mutation == 'missing_warmup': del rows[0]['warmup']
            if mutation == 'sample_plan': raw['samples'] = 4
            if mutation == 'boolean_warmup_count': raw['warmups'] = True
            if mutation == 'two_cases': raw['cases'].append(copy.deepcopy(raw['cases'][0]))
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                validate(raw)

    def test_boolean_one_is_not_an_admitted_single_warmup_plan(self):
        raw = payload()
        raw['warmups'] = True
        raw['cases'][0]['observations'].pop(1)
        with self.assertRaises(ValueError):
            cohort.validate_baseline_payload(raw, case_name=CASE, fixture_sha256=SOURCE_SHA,
                executable_sha256=EXE_SHA, source_commit=SOURCE_COMMIT,
                expected_stdout=EXPECTED, samples=3, warmups=1)

    def test_every_warmup_and_checked_checksum_is_required_including_exact_newline(self):
        for index in range(5):
            for wrong in ('wrong\n', EXPECTED.rstrip('\n'), None):
                raw = payload()
                raw['cases'][0]['observations'][index]['stdout'] = wrong
                with self.subTest(index=index, wrong=wrong), self.assertRaises(ValueError):
                    validate(raw)
        raw = payload()
        del raw['cases'][0]['observations'][0]['stdout']
        with self.assertRaises(ValueError):
            validate(raw)

    def test_explicit_missing_or_inconsistent_policy_cannot_enter_ordinary_cohort(self):
        for mutation in ('schema', 'missing_requested', 'explicit_requested', 'observed_explicit',
                         'missing_runtime', 'different_runtime'):
            raw = payload()
            if mutation == 'schema': raw['schema'] = 'scriptw-wasmtime-policy-baseline-v1'
            if mutation == 'missing_requested': del raw['requested_native_policy']
            if mutation == 'explicit_requested':
                raw['requested_native_policy'] = dict(native_mode='fast', gc_initial_mib=32,
                                                     register_allocation='single-pass')
            if mutation == 'observed_explicit':
                raw['cases'][0]['observations'][0]['actual_native_policy'] = None
            if mutation == 'missing_runtime': del raw['cases'][0]['observations'][0]['runtime_policy']
            if mutation == 'different_runtime': raw['cases'][0]['observations'][3]['runtime_policy'] = 'other'
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                validate(raw)

    def test_warm_p_hits_and_complete_nonnegative_integer_timings_are_required(self):
        for mutation in ('warmup_miss', 'checked_miss', 'invalid_cache', 'boolean_execution',
                         'negative_phase', 'missing_phase', 'wrong_total', 'boolean_phase'):
            raw = payload()
            rows = raw['cases'][0]['observations']
            if mutation == 'warmup_miss': rows[1]['module_cache'] = 'Miss'
            if mutation == 'checked_miss': rows[2]['module_cache'] = 'Miss'
            if mutation == 'invalid_cache': rows[0]['module_cache'] = 'NotAccessed'
            if mutation == 'boolean_execution': rows[0]['execution_ns'] = True
            if mutation == 'negative_phase': rows[0]['phases_ns']['module'] = -1
            if mutation == 'missing_phase': del rows[0]['phases_ns']['admission']
            if mutation == 'wrong_total': rows[0]['phases_ns']['total'] = 5
            if mutation == 'boolean_phase': rows[0]['phases_ns']['module'] = False
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                validate(raw)

    def test_corrupt_source_artifact_and_observation_records_are_rejected(self):
        for mutation in ('raw', 'cases', 'row', 'source', 'source_hash', 'file', 'executable', 'fingerprint',
                         'profile', 'p_hash', 'p_bytes', 'r_missing', 'r_hash', 'r_bytes'):
            raw = payload()
            case = raw['cases'][0]
            if mutation == 'raw': raw = []
            if mutation == 'cases': raw['cases'] = 'objects.js'
            if mutation == 'row': case['observations'][0] = None
            if mutation == 'source': raw['source_commit'] = 'other'
            if mutation == 'source_hash': case['source_sha256'] = 'f' * 64
            if mutation == 'file': case['file'] = 'integer.js'
            if mutation == 'executable': raw['executable_sha256'] = 'f' * 64
            if mutation == 'fingerprint': del raw['source_fingerprint']
            if mutation == 'profile': case['source_compilation_profile'] = 'native-host'
            if mutation == 'p_hash': case['wasm_sha256'] = 'not-a-sha'
            if mutation == 'p_bytes': case['wasm_bytes'] = True
            if mutation == 'r_missing': case['linked_runtime'] = None
            if mutation == 'r_hash': case['linked_runtime']['wasm_sha256'] = 'not-a-sha'
            if mutation == 'r_bytes': case['linked_runtime']['wasm_bytes'] = 0
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                validate(raw)

    def test_each_cohort_keeps_identical_p_r_compiler_and_runtime_policy(self):
        identity = validate(payload())[2]
        for mutation in ('p', 'p_size', 'r_key', 'r', 'r_size', 'fingerprint', 'runtime_policy'):
            raw = payload()
            case = raw['cases'][0]
            if mutation == 'p': case['wasm_sha256'] = 'f' * 64
            if mutation == 'p_size': case['wasm_bytes'] += 1
            if mutation == 'r_key': case['linked_runtime']['key'] = 'f' * 64
            if mutation == 'r': case['linked_runtime']['wasm_sha256'] = 'f' * 64
            if mutation == 'r_size': case['linked_runtime']['wasm_bytes'] += 1
            if mutation == 'fingerprint': raw['source_fingerprint'] = 'f' * 64
            if mutation == 'runtime_policy':
                for row in case['observations']: row['runtime_policy'] = 'other'
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                validate(raw, identity)
