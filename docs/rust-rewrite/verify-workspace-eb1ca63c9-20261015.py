#!/usr/bin/env python3
"""Offline current-workspace adapter around the accepted raw/Cargo reader.

The predecessor files are supplied separately by exact accepted SHA. Their
large historical payloads are never copied into each successor checkpoint.
"""
import argparse
import base64
from collections import Counter
import hashlib
import json
from pathlib import Path
import re
import types
import zlib

CORE_SHA = '81a64669037c9198ba093141e0d498034043d7139a775b6e9a6e9eec3b38751b'
PRIOR = {'default': 'e99d0dd1c41facd021fa5e21740124c7ef20b216319df59e5724843661123e49',
         'all_features': 'f5676ee569ab72c8f0a05597b1f43941de0aef692cdcc18e868636bb3fe18a87'}
CONTROL_IDS = (('lila-ir', 'lila_ir', 'lib', 'tests::labelled_block_and_case_environments_keep_only_direct_lexical_label_items'), ('lila-ir', 'lila_ir', 'lib', 'tests::labelled_case_initialization_omits_chains_without_changing_annex_b_copies'), ('lila-ir', 'lila_ir', 'lib', 'tests::labelled_chains_keep_script_and_function_top_level_var_hoisting'), ('lila-ir', 'lila_ir', 'lib', 'tests::labelled_lexical_selector_stops_at_a_statement_instead_of_unwrapping_labels'), ('lila-engine', 'aot_language', 'test', 'aot_callable_entry_roles::labelled_chains_do_not_create_block_or_case_bindings'), ('lila-engine', 'aot_language', 'test', 'aot_callable_entry_roles::labelled_chains_do_not_invent_closure_aliases_or_finite_eval_bindings'), ('lila-engine', 'aot_language', 'test', 'aot_callable_entry_roles::labelled_single_lexical_items_and_top_level_chains_keep_hoisted_functions'), ('lila-engine', 'aot_language', 'test', 'aot_callable_entry_roles::unchanged_pinned_labelled_function_syntax_keeps_both_top_level_names'), ('lila-ir', 'lila_ir', 'lib', 'regexp::source_escape::source_context_tests::line_terminator_escapes_reuse_an_existing_odd_backslash'), ('lila-ir', 'lila_ir', 'lib', 'regexp::source_escape::source_context_tests::slashes_follow_lexical_class_and_backslash_context'), ('lila-engine', 'aot_regexp', 'test', 'aot_regexp_finite_source::literal_source_keeps_character_class_slashes_and_backslash_spellings'), ('lila-engine', 'aot_regexp', 'test', 'aot_regexp_finite_source::source_getter_preserves_unicode_set_slashes_after_nested_class_closes'), ('lila-ir', 'lila_ir_integration', 'test', 'ir_cases::finite_regexp_source::literal_and_resolved_constructor_sources_prepare_escaped_script_candidates'), ('lila-engine', 'aot_regexp', 'test', 'aot_regexp_finite_source::source_roundtrips_keep_slash_parity_line_terminators_and_raw_surrogates'), ('lila-ir', 'lila_ir', 'lib', 'tests::labelled_case_chains_do_not_reserve_activation_cells_visible_to_direct_eval'), ('lila-ir', 'lila_ir', 'lib', 'tests::labelled_chains_do_not_reserve_activation_cells_visible_to_direct_eval'), ('lila-engine', 'aot_language', 'test', 'aot_callable_entry_roles::labelled_nonlexical_chains_do_not_shadow_initialized_bindings_in_direct_eval'))
PRIOR_HEAD = '8e9a8c8210f99d8d14b4267461ff42079b603c00'
ROOT = '/workspace/porffor-next/'
SCHEMA = 'portable-current-workspace-retention-v1'


def require(value, message):
    if not value:
        raise ValueError(message)


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def packed(raw):
    return {'encoding': 'base64(zlib(original bytes))', 'bytes': len(raw), 'sha256': digest(raw),
            'data': base64.b64encode(zlib.compress(raw, 9)).decode()}


def load_core(raw):
    require(digest(raw) == CORE_SHA, 'Accepted raw/Cargo reader source changed')
    module = types.ModuleType('accepted_workspace_raw_core')
    module.__file__ = '<sha-bound-accepted-workspace-core>'
    exec(compile(raw, module.__file__, 'exec'), vars(module))
    return module


def qualified(mapping):
    return {(*scope, name) for scope, outcomes in mapping.items() for name, value in outcomes.items() if value == 'ok'}


def retention(prior, current, disabled=()):
    required = qualified(prior)
    current_passes = qualified(current)
    exempt = set(disabled)
    require(exempt <= required, 'Default-only exemption is outside exact prior passes')
    missing = required - current_passes - exempt
    return {'required': len(required), 'observed_current_successes': len(required & current_passes),
            'same_source_default_only': len(exempt),
            'regressions': [list(value) for value in sorted(missing)], 'passed': not missing}


def prior_maps(mode, documents, core, cache):
    if mode in cache:
        return cache[mode]
    raw = documents[mode]
    require(digest(raw) == PRIOR[mode], 'Accepted 8e predecessor SHA changed')
    document = json.loads(raw)
    require(document['compiler_revision'] == PRIOR_HEAD and document['feature_mode'] == mode
            and document['whole_workspace_passed'] is True, 'Accepted predecessor source/mode differs')
    _, _, read = core.raw_layers(document)
    audit = json.loads(core.unpack(document['exact_audit']))
    rows = core.scope_maps(audit, read)
    result = {scope: value['verdicts'] for scope, value in rows.items()}
    require(len(result) == (107 if mode == 'default' else 108)
            and len(qualified(result)) == (9161 if mode == 'default' else 9254), 'Predecessor exact raw count differs')
    cache[mode] = result
    return result


CONTROL_REGISTRATION = [{'scope': ['lila-ir', 'lila_ir', 'lib'], 'id': 'tests::labelled_block_and_case_environments_keep_only_direct_lexical_label_items', 'source_path': 'crates/lila-ir/src/tests/annex_b.rs', 'registration': 'new'}, {'scope': ['lila-ir', 'lila_ir', 'lib'], 'id': 'tests::labelled_case_initialization_omits_chains_without_changing_annex_b_copies', 'source_path': 'crates/lila-ir/src/tests/annex_b.rs', 'registration': 'new'}, {'scope': ['lila-ir', 'lila_ir', 'lib'], 'id': 'tests::labelled_chains_keep_script_and_function_top_level_var_hoisting', 'source_path': 'crates/lila-ir/src/tests/annex_b.rs', 'registration': 'new'}, {'scope': ['lila-ir', 'lila_ir', 'lib'], 'id': 'tests::labelled_lexical_selector_stops_at_a_statement_instead_of_unwrapping_labels', 'source_path': 'crates/lila-ir/src/tests/annex_b.rs', 'registration': 'new'}, {'scope': ['lila-engine', 'aot_language', 'test'], 'id': 'aot_callable_entry_roles::labelled_chains_do_not_create_block_or_case_bindings', 'source_path': 'crates/lila-engine/tests/aot_language/aot_callable_entry_roles.rs', 'registration': 'new'}, {'scope': ['lila-engine', 'aot_language', 'test'], 'id': 'aot_callable_entry_roles::labelled_chains_do_not_invent_closure_aliases_or_finite_eval_bindings', 'source_path': 'crates/lila-engine/tests/aot_language/aot_callable_entry_roles.rs', 'registration': 'new'}, {'scope': ['lila-engine', 'aot_language', 'test'], 'id': 'aot_callable_entry_roles::labelled_single_lexical_items_and_top_level_chains_keep_hoisted_functions', 'source_path': 'crates/lila-engine/tests/aot_language/aot_callable_entry_roles.rs', 'registration': 'new'}, {'scope': ['lila-engine', 'aot_language', 'test'], 'id': 'aot_callable_entry_roles::unchanged_pinned_labelled_function_syntax_keeps_both_top_level_names', 'source_path': 'crates/lila-engine/tests/aot_language/aot_callable_entry_roles.rs', 'registration': 'new'}, {'scope': ['lila-ir', 'lila_ir', 'lib'], 'id': 'regexp::source_escape::source_context_tests::line_terminator_escapes_reuse_an_existing_odd_backslash', 'source_path': 'crates/lila-ir/src/regexp/source_escape.rs', 'registration': 'new'}, {'scope': ['lila-ir', 'lila_ir', 'lib'], 'id': 'regexp::source_escape::source_context_tests::slashes_follow_lexical_class_and_backslash_context', 'source_path': 'crates/lila-ir/src/regexp/source_escape.rs', 'registration': 'new'}, {'scope': ['lila-engine', 'aot_regexp', 'test'], 'id': 'aot_regexp_finite_source::literal_source_keeps_character_class_slashes_and_backslash_spellings', 'source_path': 'crates/lila-engine/tests/aot_regexp/aot_regexp_finite_source.rs', 'registration': 'new'}, {'scope': ['lila-engine', 'aot_regexp', 'test'], 'id': 'aot_regexp_finite_source::source_getter_preserves_unicode_set_slashes_after_nested_class_closes', 'source_path': 'crates/lila-engine/tests/aot_regexp/aot_regexp_finite_source.rs', 'registration': 'new'}, {'scope': ['lila-ir', 'lila_ir_integration', 'test'], 'id': 'ir_cases::finite_regexp_source::literal_and_resolved_constructor_sources_prepare_escaped_script_candidates', 'source_path': 'crates/lila-ir/tests/finite_regexp_source.rs', 'registration': 'strengthened-existing'}, {'scope': ['lila-engine', 'aot_regexp', 'test'], 'id': 'aot_regexp_finite_source::source_roundtrips_keep_slash_parity_line_terminators_and_raw_surrogates', 'source_path': 'crates/lila-engine/tests/aot_regexp/aot_regexp_finite_source.rs', 'registration': 'strengthened-existing'}, {'scope': ['lila-ir', 'lila_ir', 'lib'], 'id': 'tests::labelled_case_chains_do_not_reserve_activation_cells_visible_to_direct_eval', 'source_path': 'crates/lila-ir/src/tests/annex_b.rs', 'registration': 'new'}, {'scope': ['lila-ir', 'lila_ir', 'lib'], 'id': 'tests::labelled_chains_do_not_reserve_activation_cells_visible_to_direct_eval', 'source_path': 'crates/lila-ir/src/tests/annex_b.rs', 'registration': 'new'}, {'scope': ['lila-engine', 'aot_language', 'test'], 'id': 'aot_callable_entry_roles::labelled_nonlexical_chains_do_not_shadow_initialized_bindings_in_direct_eval', 'source_path': 'crates/lila-engine/tests/aot_language/aot_callable_entry_roles.rs', 'registration': 'new'}]


def controls(observations, registration):
    require(registration == CONTROL_REGISTRATION, 'Exact required control registration changed')
    keys = [(*row['scope'], row['id']) for row in registration]
    require(set(keys) == set(CONTROL_IDS) and len(keys) == len(set(keys)) == 17, 'Expected the exact 15 new and two strengthened controls')
    rows = [{**row, 'actual_outcome': observations.get(tuple(row['scope']), {}).get(row['id'])}
            for row in registration]
    require(all(row['actual_outcome'] == 'ok' for row in rows), 'A required RegExp/label control has no actual current PASS')
    return rows


def resource_policy(owner, closed):
    command=closed['command']
    require(command.count('--cloud-worker-cpus') == 1 and command[command.index('--cloud-worker-cpus')+1] == 'auto'
            and '--cloud-cpus' not in command and '--cloud' in command, 'Actual outer cloud worker policy changed')
    require(owner['scheduling']['native_cpus'] == [0,1,2,3] and owner['scheduling']['test_processes'] == 4
            and owner['scheduling']['cargo_jobs'] == owner['scheduling']['test_threads_per_process'] == 1,
            'Actual 4-CPU / serial Cargo / one-thread policy changed')
    require(owner['isolated_test_packages'] == ['lila-intl']
            and set(owner['partition_native_targets']) == {'lila-engine:aot_intl','lila-engine:lila_engine','lila-engine:aot_async'}
            and owner['partition_cli'] is True and owner['scheduling']['strip_test_debug'] is True,
            'Full workspace isolation/partition/shared-runtime policy changed')


def verify_current(document, prior_documents, default_document=None, *, cache=None):
    require(document['schema'] == SCHEMA and document['mode'] in ('default', 'all_features'), 'Unknown current checkpoint')
    cache = {} if cache is None else cache
    core_raw = base64.b64decode(document['raw_reader_source']['base64'], validate=True)
    require(document['raw_reader_source']['sha256'] == CORE_SHA, 'Raw reader metadata changed')
    core = load_core(core_raw)
    head, mode = document['head'], document['mode']
    require(re.fullmatch('[0-9a-f]{40}', head), 'Exact current source HEAD required')
    core.HEAD = head
    audit_raw = core.unpack(document['base_audit'])
    require(digest(audit_raw) == document['base_audit_ref']['sha256'], 'Actual base audit SHA changed')
    audit = json.loads(audit_raw)
    all_features = mode == 'all_features'
    core.accepted_audit(audit, all_features)
    store = core.RawStore(document)
    store.verify_all()
    binaries = {row['observed_path']: row for row in document['observed_binary_bindings']}
    require(len(binaries) == len(document['observed_binary_bindings']), 'Duplicate producer/image closure binding')
    for path, row in binaries.items():
        require(row['payload_embedded'] is False and all(audit['bound_files'][path][key] == row[key] for key in ('sha256','bytes')),
                'Producer/image identity differs from the actual live audit')
    observations, metadata, runs = core.read_current_scopes(audit, store.get, binaries, all_features, head=head)
    core.verify_closed_union(audit, runs, store.get, all_features)
    snapshot = document['prepared_source']
    require(snapshot['head'] == head and snapshot['source_fences']['before'] == snapshot['source_fences']['after'],
            'Prepared source/index fence differs')
    require(audit['read_only_verifier_source_index_fences']['before'] == snapshot['source_fences']['before']
            and audit['read_only_verifier_source_index_fences']['after'] == snapshot['source_fences']['after'],
            'Actual audit source/index differs from prepared source')
    for row in snapshot['files']:
        raw = store.get(row['path'], row['sha256'])
        require(len(raw) == row['bytes'] and row['path'] == ROOT + row['relative_path'], 'Exact source snapshot differs')
    scripts = {row['relative_path']: row for row in snapshot['files']}
    require(len(scripts) == len(snapshot['files']), 'Duplicate source file binding')
    for control in snapshot['controls']:
        require(control['source_path'] in scripts, 'Required control source was omitted')
        row=scripts[control['source_path']]
        source=store.get(row['path'],row['sha256']).decode()
        require(len(re.findall(r'#\[test\]\s*fn\s+'+re.escape(control['id'].split('::')[-1])+r'\s*\(',source)) == 1,
                'Required current control lacks its exact source definition')
    require(scripts['scripts/run_workspace_tests.py']['sha256'] == 'c644a978840767cbafd99378aff26e8e059af1713855373cec9bbd5cc01ac591'
            and scripts['scripts/workspace_test_partitions.py']['sha256'] == core.PARTITION_DRIVER_SHA,
            'Actual native runner/partition source changed')
    for run, owner in zip(audit['closed_actual_runs'], runs):
        closed = json.loads(store.get(run['outer_owner']['path'], run['outer_owner']['sha256']))
        resource_policy(owner,closed)
    old_default = prior_maps('default', prior_documents, core, cache)
    disabled = set()
    if all_features:
        require(default_document is not None and digest(default_document) == document['same_source_default']['sha256'],
                'Exact accepted same-source default checkpoint required')
        default = json.loads(default_document)
        require(default['head'] == head and default['mode'] == 'default', 'Default checkpoint source/mode differs')
        verified_default = verify_current(default, prior_documents, cache=cache)
        require(verified_default['whole_workspace_passed'], 'Same-source default did not pass')
        default_audit = json.loads(core.unpack(default['base_audit']))
        registry = snapshot['default_only_registry']
        disabled = set(core.qualified_default_ids(audit, default_audit, registry, observations, metadata, store.get))
        require(disabled == qualified(old_default) - qualified(prior_maps('all_features', prior_documents, core, cache))
                and len(disabled) == 4, 'Default-only exemption must be exactly the four prior native tests')
    checks = {'prior_default_9161': retention(old_default, observations, disabled)}
    if all_features:
        checks['prior_all_features_9254'] = retention(prior_maps('all_features', prior_documents, core, cache), observations)
    require(all(row['passed'] for row in checks.values()), 'An accepted 8e native PASS regressed or disappeared')
    control_rows = controls(observations, snapshot['controls'])
    require(document['retention'] == checks and document['controls'] == control_rows,
            'Checkpoint retention/control projections differ from raw outcomes')
    counts = dict(Counter(value for scope in observations.values() for value in scope.values()))
    require(counts == document['native_counts'] == audit['actual_native_counts'] and set(counts) == {'ok'},
            'Literal native result counts differ')
    require(document['task_acceptance'] is document['ordinary_cli_acceptance'] is document['full_pin_acceptance'] is False,
            'Workspace proof cannot grant unrelated acceptance')
    return {'verified': True, 'head': head, 'mode': mode, 'native_counts': counts,
            'metadata_scopes': len(observations), 'retention': checks, 'required_controls': len(control_rows),
            'whole_workspace_passed': True, 'task_acceptance': False}


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('checkpoint',type=Path)
    parser.add_argument('--prior-default',type=Path,required=True)
    parser.add_argument('--prior-all-features',type=Path)
    parser.add_argument('--same-source-default',type=Path)
    args=parser.parse_args()
    predecessors={'default':args.prior_default.read_bytes()}
    if args.prior_all_features:predecessors['all_features']=args.prior_all_features.read_bytes()
    result=verify_current(json.loads(args.checkpoint.read_bytes()),predecessors,
                          args.same_source_default.read_bytes() if args.same_source_default else None)
    print(json.dumps(result,indent=2))


if __name__=='__main__':main()
