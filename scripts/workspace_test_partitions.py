"""Run every qualified native libtest ID once in disjoint processes."""
import argparse
import hashlib
import json
import os
import re
import subprocess
import sys
import time
from pathlib import Path

from workspace_test_schedule import native_stall_seconds


# Approximate watcher intervals from the complete 855-ID CLI run on 2026-10-20.
# Three units per second keep these comparable to the ordinary-test weight.
# See docs/rust-rewrite/contracts/native-partition-costs.md. These are estimates,
# never selectors: renamed/new tests retain the fallback weights below.
CLI_OBSERVED_SECONDS = {
    'frontend::test262_run_writes_summary': 114,
    'frontend_test262_subset::test262_wasm_backend_runs_supported_fixture_subset': 498,
    'frontend::test262_backlog_and_snapshot_compare_read_completed_matrix_snapshots': 233,
    'frontend::test262_report_all_aggregates_fixture_suite': 118,
    'intl::calendar_manifest_v3_build_export_and_run_share_the_sdk_projected_service_graph': 58,
    'intl::combined_projection_cli_build_matches_sdk_and_runs_both_selected_services': 71,
    'intl::custom_intl_profile_reaches_ordinary_cli_build_script_and_module_run': 76,
    'intl::datetime_cli_projection_matches_sdk_and_runs_six_selected_components': 180,
    'intl::duration_cli_projection_matches_sdk_and_runs_four_selected_components': 143,
    'intl::number_cli_projection_matches_sdk_and_runs_the_coupled_public_services': 130,
    'intl::projected_list_cli_build_and_module_run_use_the_actual_sdk_projection': 43,
    'intl::segmenter_cli_projection_matches_sdk_and_runs_eight_selected_components': 159,
    'frontend::test262_report_groups_failures_by_bucket': 90,
    'frontend::test262_triage_and_failure_details_read_completed_matrix_snapshots': 91,
    'intl::calendar_manifest_v3_cli_matches_sdk_and_preserves_global_available_values': 58,
    'intl::collator_cli_projection_matches_sdk_and_runs_seven_selected_components': 119,
    'intl::currency_manifest_v2_build_and_export_use_the_actual_sdk_data_before_loading_source': 33,
    'intl::custom_manifest_cli_matches_the_actual_sdk_graph_and_rejects_before_source_loading': 64,
    'intl::display_names_cli_projection_matches_sdk_and_composes_all_three_filters': 95,
    'intl::named_zone_manifest_v5_build_export_and_unavailable_data_follow_actual_sdk_dependencies': 81,
    'intl::numbering_manifest_v4_build_export_and_run_consume_the_same_paired_sdk_selection': 44,
    'intl::service_manifest_v6_cli_build_export_and_run_share_the_checked_sparse_sdk_owner': 66,
}


parser = argparse.ArgumentParser()
parser.add_argument('--executable', required=True, type=Path)
parser.add_argument('--inventory', required=True, type=Path)
parser.add_argument('--output', required=True, type=Path)
parser.add_argument('--cpus', required=True, nargs='+', type=int)
parser.add_argument('--include-ignored', action='store_true', required=True)
parser.add_argument('--inventory-only', action='store_true')
parser.add_argument('--qualified-target', default='scriptw-cli:cli')
parser.add_argument('--stall-seconds', type=int, choices=(900, 3600))
args = parser.parse_args()
package, separator, target = args.qualified_target.partition(':')
assert separator and package and target and ':' not in target, 'An exact package:target is required'
stock_stall = native_stall_seconds(package)
assert args.stall_seconds in (None, stock_stall), 'The original native watcher allowance cannot change'
args.stall_seconds = stock_stall
root = Path.cwd()
assert 1 <= len(args.cpus) <= 4
assert len(set(args.cpus)) == len(args.cpus)
assert set(args.cpus) <= os.sched_getaffinity(0)
args.output.mkdir(parents=True, exist_ok=False)


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def names(text):
    return [line.removesuffix(': test') for line in text.splitlines()
            if line.endswith(': test')]


def outcomes(text):
    starts = list(re.finditer(r'^test (.+?) \.\.\. ', text, re.M))
    results = {}
    for index, start in enumerate(starts):
        end = starts[index + 1].start() if index + 1 < len(starts) else len(text)
        block = text[start.end():end].split('\nfailures:', 1)[0]
        terminal = re.findall(r'^(ok|FAILED|ignored)$', block, re.M)
        assert len(terminal) == 1, (start.group(1), terminal)
        name = start.group(1).removesuffix(' - should panic')
        assert name not in results, name
        results[name] = terminal[0]
    summaries = re.findall(
        r'^test result: .*? (\d+) passed; (\d+) failed; (\d+) ignored;', text, re.M)
    assert len(summaries) == 1, summaries
    counts = {key: sum(value == terminal for value in results.values())
              for key, terminal in [('passed', 'ok'), ('failed', 'FAILED'), ('ignored', 'ignored')]}
    assert tuple(counts.values()) == tuple(map(int, summaries[0]))
    assert counts['ignored'] == 0
    return results, counts


def weight(name):
    # Scheduling estimates are never used to select, drop or retry a test.
    if name in CLI_OBSERVED_SECONDS:
        return 3 * CLI_OBSERVED_SECONDS[name]
    # This measured Engine outlier dominates an otherwise balanced partition.
    if name == ('runtime_regexp_compiler_tests::raw_utf16::'
                'literal_prepared_and_static_constructor_patterns_share_actual_utf16_units'):
        return 900
    if name.startswith('frontend_test262_subset::'):
        return 800
    if name.startswith('intl::'):
        return 25
    if name.startswith('frontend::') or name.startswith('known_failures::'):
        return 1
    return 3


def partition_groups(original, count):
    assert original and len(original) == len(set(original))
    assert 1 <= count <= min(4, len(original)), 'Every admitted native partition needs an ID'
    groups = [[] for _ in range(count)]
    loads = [0 for _ in range(count)]
    for name in sorted(original, key=lambda item: (-weight(item), item)):
        index = min(range(len(groups)), key=lambda group: (loads[group], group))
        groups[index].append(name)
        loads[index] += weight(name)
    assert sorted(name for group in groups for name in group) == sorted(original)
    assert all(groups)
    return groups


def partition_label(output, index):
    # A target basename repeats across workspace runs, while watched logs live
    # in one shared directory. Bind each label to its complete output owner.
    output = Path(output).resolve()
    owner = hashlib.sha256(os.fsencode(output)).hexdigest()
    prefix = re.sub(r'[^A-Za-z0-9_.-]', '-', output.name)[:64] or 'native'
    return f'{prefix}-{owner}-partition-{index}'


original = names(args.inventory.read_text())
assert original and len(original) == len(set(original))
checked = subprocess.run([str(args.executable.resolve()), '--list'],
                         capture_output=True, text=True)
assert checked.returncode == 0
assert names(checked.stdout) == original, 'The original complete native inventory changed'
full_inventory = args.output / 'complete.inventory.log'
full_inventory.write_text(checked.stdout + checked.stderr)
groups = partition_groups(original, len(args.cpus))
executable_sha = digest(args.executable)
receipt = {
    'policy': 'Complete logical target: every original qualified ID exactly once in disjoint native libtest processes, each on one inherited CPU and one test thread; original bodies and deadlines unchanged; zero ignored required.',
    'driver_sha256': digest(Path(__file__)),
    'qualified_target': args.qualified_target, 'stall_seconds': args.stall_seconds,
    'inherited_retained_modules': {
        'entries_per_process': os.environ.get('SCRIPTW_MODULE_MEMORY_CACHE_ENTRIES'),
        'image_bytes_per_process': os.environ.get('SCRIPTW_MODULE_MEMORY_CACHE_LIMIT_BYTES'),
        'policy': 'Inherited environment; each child retains the existing limited_verification cloud ceiling. No cache or memory limit changed.'},
    'executable': str(args.executable), 'executable_sha256': executable_sha,
    'original_inventory': str(args.inventory), 'original_inventory_sha256': digest(args.inventory),
    'complete_inventory': str(full_inventory), 'complete_inventory_sha256': digest(full_inventory),
    'complete_test_count': len(original), 'cpus': args.cpus, 'partitions': [],
    'source_authority': 'Outer workspace receipt binds the exact Cargo-selected executable and immutable source fence.',
}


def save():
    (args.output / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')


# Validate every actual filtered inventory before starting any native test body.
for index, (cpu, group) in enumerate(zip(args.cpus, groups)):
    group.sort()
    command = [str(args.executable.resolve()), '--exact', *group]
    listed = subprocess.run([*command, '--list'], capture_output=True, text=True)
    inventory = args.output / f'partition-{index}.inventory.log'
    inventory.write_text(listed.stdout + listed.stderr)
    assert listed.returncode == 0 and names(listed.stdout) == group
    receipt['partitions'].append({
        'index': index, 'cpu': cpu, 'ids': group,
        'inventory_log': str(inventory), 'inventory_sha256': digest(inventory),
        'command': [*command, '--test-threads=1', '--include-ignored'],
    })
save()
if args.inventory_only:
    receipt['state'] = 'inventory-only; no native test body executed or verdict claimed'
    save()
    print(json.dumps({'complete_test_count': len(original),
                      'partition_counts': [len(group) for group in groups],
                      'exact_disjoint_union': True}, indent=2))
    raise SystemExit(0)
active = []
started = time.monotonic()
for row in receipt['partitions']:
    label = partition_label(args.output, row['index'])
    log = args.output / f'partition-{row["index"]}.launcher.log'
    stream = log.open('w')
    launch = ['taskset', '-c', str(row['cpu']), './scripts/run-watched.sh',
              '--label', label, '--stall', str(args.stall_seconds), '--poll', '1', '--',
              'python3', '-B', 'scripts/limited_verification.py', '--cloud', '--', *row['command']]
    row['launch_command'] = launch
    row['transcript_log'] = str(Path('target/watched') / (label + '.log'))
    row['state'] = 'running'
    active.append((subprocess.Popen(launch, stdout=stream, stderr=subprocess.STDOUT), stream, row))
save()
last_update = 0.0
while active:
    now = time.monotonic()
    if now - last_update >= 20:
        print('PARTITION PROGRESS', json.dumps({row['index']: Path(row['transcript_log']).stat().st_size
              for _, _, row in active if Path(row['transcript_log']).exists()}), flush=True)
        last_update = now
    pending = []
    for process, stream, row in active:
        status = process.poll()
        if status is None:
            pending.append((process, stream, row))
            continue
        stream.close()
        row['exit_code'] = status
        transcript = Path(row['transcript_log'])
        if transcript.exists():
            row['transcript_sha256'] = digest(transcript)
        try:
            actual, counts = outcomes(transcript.read_text())
            assert set(actual) == set(row['ids'])
            row['native_outcomes'] = actual
            row['counts'] = counts
            row['inventory_matches'] = True
            row['state'] = 'passed' if status == 0 and counts['failed'] == 0 else 'failed'
        except (AssertionError, OSError) as error:
            row['state'] = 'incomplete'
            row['inventory_matches'] = False
            row['validation_error'] = str(error)
        print('PARTITION END', row['index'], row['state'], row.get('counts'), flush=True)
        save()
    active = pending
    if active:
        time.sleep(.1)

assert digest(args.executable) == executable_sha, 'Native artifact changed during execution'
receipt['seconds'] = round(time.monotonic() - started, 3)
receipt['inventory_matches'] = all(row['inventory_matches'] for row in receipt['partitions'])
if receipt['inventory_matches']:
    actual = {name: result for row in receipt['partitions'] for name, result in row['native_outcomes'].items()}
    assert len(actual) == len(original) and set(actual) == set(original)
    receipt['counts'] = {key: sum(row['counts'][key] for row in receipt['partitions'])
                         for key in ('passed', 'failed', 'ignored')}
save()
# A composite view retains all native result lines, with one recomputed summary.
# Original complete per-process transcripts are retained and hash-bound above.
for row in receipt['partitions']:
    transcript = Path(row['transcript_log'])
    if transcript.exists():
        text = re.sub(r'^test result: .*\n?', '', transcript.read_text(), flags=re.M)
        print(text, end='\n', flush=True)
if receipt['inventory_matches']:
    counts = receipt['counts']
    state = 'ok' if all(row['state'] == 'passed' for row in receipt['partitions']) else 'FAILED'
    print(f'test result: {state}. {counts["passed"]} passed; {counts["failed"]} failed; {counts["ignored"]} ignored; 0 measured; 0 filtered out;', flush=True)
raise SystemExit(not receipt['inventory_matches'] or any(row['state'] != 'passed' for row in receipt['partitions']))
