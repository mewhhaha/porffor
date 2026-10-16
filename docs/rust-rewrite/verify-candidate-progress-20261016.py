#!/usr/bin/env python3
"""Verify the portable candidate checkpoint without private files or execution."""
import argparse
import base64
import collections
import hashlib
import json
from pathlib import Path
import re
import zlib

EXPECTED_FAILURES = {
    'ir_unit': {
        'lowering::dynamic_source::native_global_script_tests::opaque_realm_script_source_retains_the_actual_callable_and_runtime_invocation',
        'lowering::dynamic_source::native_global_script_tests::realm_script_spelling_cannot_admit_a_replacement_ordinary_callable_as_source',
        'modules::graph::tests::source_phase_exports::dynamic_source_jobs_retain_native_source_cells_without_evaluation',
        'modules::namespace::tests::a_source_phase_binding_has_no_namespace_or_source_cell',
        'tests::realm_eval_script_is_a_test262_only_typed_host_capability',
    },
    'ir_integration': {
        'ir_cases::indirect_eval_admission::realm_script_keeps_its_distinct_source_conversion_requirement',
        'ir_cases::module_unit_structure::module_unit_has_one_private_owner_and_narrow_public_facade',
    },
}


def verify(proof):
    def require(value, message):
        if not value:
            raise ValueError(message)

    require(proof['schema'] == 'lila-private-candidate-progress-20261016-v1', 'Wrong proof schema')
    blobs = {}
    for row in proof['raw_evidence']:
        require(row['encoding'] == 'zlib+base64' and row['bytes'] <= 40 * 1024 * 1024, 'Invalid raw evidence bound')
        decoder = zlib.decompressobj()
        data = decoder.decompress(base64.b64decode(row['data'], validate=True), row['bytes'] + 1)
        require(decoder.eof and not decoder.unused_data and len(data) == row['bytes'], 'Incomplete or oversized raw evidence')
        require(hashlib.sha256(data).hexdigest() == row['sha256'], 'Raw evidence digest differs')
        require(row['sha256'] not in blobs or blobs[row['sha256']] == data, 'Conflicting evidence digest')
        blobs[row['sha256']] = data

    def raw(reference):
        data = blobs[reference['sha256']]
        require(len(data) == reference['bytes'], 'Referenced byte count differs')
        return data

    manifest = json.loads(raw(proof['candidate']['manifest_raw']))
    require(proof['candidate']['manifest_raw']['sha256'] == proof['candidate']['source_manifest_sha256'], 'Wrong selected manifest bytes')
    require(manifest['expected_runtime_layout'] == proof['candidate']['runtime_layout'] == 9, 'Runtime layout differs')
    failures = json.loads(raw(proof['baseline_nonpasses']))
    require(failures['complete'] and not failures['all_zero_acceptance'], 'Historical baseline was relabelled')
    former = {r['execution_id']: r['literal_failure'] for r in failures['rows']}

    def owner(name, passed):
        result = json.loads(raw(proof['owners'][name]))
        require(result['complete'] and result['passed'] is passed and result['source_unchanged'], 'Literal owner status differs')
        require(result['source_before'] == result['source_after'], 'Actual source fence differs')
        require(result['protected_before'] == result['protected_after'], 'Actual protected input census differs')
        if name != 'generic96_baseline':
            source = result.get('source_manifest') or json.loads(raw(result['plan']))['source_manifest']
            require(source['sha256'] == proof['candidate']['source_manifest_sha256'], 'Candidate source identity differs')
        for command in result['commands']:
            require(command['complete'] and command.get('pid') and command.get('start_tick'), 'Actual command process evidence missing')
            for key in ['stdout', 'stderr']:
                raw(command[key])
        return result

    cases = owner('real_test262_six', True)
    require(cases['literal_all_requested_success'] and cases['execution_coverage_complete'], 'Incomplete six-case replay')
    require([c['id'] for c in cases['commands']] == ['json2', 'module2', 'reflect2'], 'Wrong real cohorts')
    actual = []
    for cohort in cases['cohorts']:
        require(cohort['complete'] and cohort['passed'] and cohort['all_requested_ID_outcomes_known'], 'Unknown cohort outcome')
        require(cohort['total'] == 2 and not cohort['failure_rows'], 'Wrong cohort scope')
        require(cohort['compiler_identity']['source_fingerprint'] == proof['candidate']['compiler_fingerprint'], 'Replay compiler identity differs')
        require(cohort['counts_per_outcome'] == {'Bug': 0, 'Crash': 0, 'NotImplemented': 0, 'Success': 2}, 'Non-success cohort')
        rows = cohort['outcome_rows']
        require(all(r['outcome'] == 'Success' and r['failure'] is None for r in rows), 'A real case failed')
        require(sorted(r['test_id'] for r in rows) == sorted(cohort['success_ids']), 'Cohort IDs differ')
        actual += rows
        for key in ['native_stdout', 'native_json', 'native_text']:
            raw(cohort[key])
    require(len(actual) == 6 and sorted(actual, key=lambda x: x['test_id']) == proof['summary']['real_test262_six'], 'Six exact outcomes differ')
    require(all(r['test_id'] in former and former[r['test_id']]['outcome'] != 'Success' for r in actual), 'A claimed repaired case was not a literal old failure')

    baseline = owner('generic96_baseline', True)
    current = owner('generic96', True)
    require(current['diagnostic']['profile'] == 'Conformance' and current['diagnostic']['same_profile_callback_comparison'], 'Comparison profile differs')
    require(current['diagnostic']['baseline']['sha256'] == proof['owners']['generic96_baseline']['sha256'], 'Wrong measured predecessor')
    records = []
    for result in [baseline, current]:
        record, = [r for r in result['diagnostic']['records'] if r['protocol'] == 'generic-eval-trace-result-v1']
        require(record['evaluations'] == record['actual_runtime_source_catalog_misses'] == 96 and record['iterations_per_family'] == 32, 'Wrong diagnostic scope')
        require(record['timeout_ms'] == 60000 and len(record['steps']) == 96, 'Diagnostic deadline or step inventory differs')
        records.append(record)
    before, after = records
    comparison = proof['summary']['generic96']
    require(after['compiler_fingerprint'] == proof['candidate']['compiler_fingerprint'], 'Diagnostic compiler differs')
    require(comparison['before_execution_and_completion_ns'] == before['execution_and_completion_ns'] and comparison['after_execution_and_completion_ns'] == after['execution_and_completion_ns'], 'Measured callback interval differs')
    require(comparison['ratio'] == before['execution_and_completion_ns'] / after['execution_and_completion_ns'], 'Speed ratio differs')
    events = current['diagnostic']['native_template_events']
    require(collections.Counter(e['cache'] for e in events) == {'hit': 94, 'miss': 2}, 'Template events differ')
    require(comparison['cold_module_compile_or_lookup_ns'] == after['module_compile_or_lookup_ns'], 'Cold compile interval omitted')

    for key, totals in [('ir_unit', (1577, 5)), ('ir_integration', (732, 2))]:
        result = owner(key, False)
        command = result['commands'][-1]
        require(command['exit_code'] == 101 and all(c['exit_code'] == 0 for c in result['commands'][:-1]), 'Literal failing test command differs')
        text = raw(command['stdout']).decode()
        passed = re.findall(r'^test (\S+)(?: - should panic)? \.\.\. ok$', text, re.M)
        failed = re.findall(r'^test (\S+) \.\.\. FAILED$', text, re.M)
        require(len(passed) == totals[0] and len(failed) == totals[1] and set(failed) == EXPECTED_FAILURES[key], 'Actual IR outcomes or failed selectors differ')
        inventory = raw(result['commands'][-2]['stdout']).decode()
        names = re.findall(r'^(.+): test$', inventory, re.M)
        require(len(names) == len(set(names)) == sum(totals) and set(names) == set(passed + failed), 'IR actual inventory coverage differs')
        require(f'test result: FAILED. {totals[0]} passed; {totals[1]} failed; 0 ignored;' in text, 'Literal libtest failure summary missing')
        require(proof['summary'][key] == {'passed': totals[0], 'failed': totals[1], 'ignored': 0, 'failed_ids': sorted(failed)}, 'Public IR summary differs')

    pending = proof['acceptance']
    require(pending == {'private_candidate_only': True, 'published_source_identity_verified': False, 'ordinary_Cargo_acceptance': False, 'full_suite_acceptance': False, 'task_closed': False, 'remaining_unit_harnesses_unrun_at_capture': 5, 'remaining_integration_harnesses_unrun_at_capture': 13, 'seven_IR_assertion_successors_executed': False}, 'Unexecuted acceptance was changed')
    return {'verified': True, 'real_Test262_Success': 6, 'generic96_ratio': comparison['ratio'], 'IR_unit': [1577, 5], 'IR_integration': [732, 2], 'full_suite_acceptance': False}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('proof')
    args = parser.parse_args()
    print(json.dumps(verify(json.loads(Path(args.proof).read_text())), indent=2))
