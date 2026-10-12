"""Run every inventoried CLI libtest ID once in disjoint native processes."""
import argparse
import hashlib
import json
import os
import re
import subprocess
import sys
import time
from pathlib import Path


parser = argparse.ArgumentParser()
parser.add_argument('--executable', required=True, type=Path)
parser.add_argument('--inventory', required=True, type=Path)
parser.add_argument('--output', required=True, type=Path)
parser.add_argument('--cpus', required=True, nargs='+', type=int)
parser.add_argument('--include-ignored', action='store_true', required=True)
parser.add_argument('--inventory-only', action='store_true')
args = parser.parse_args()
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


original = names(args.inventory.read_text())
assert original and len(original) == len(set(original))
checked = subprocess.run([str(args.executable.resolve()), '--list'],
                         capture_output=True, text=True)
assert checked.returncode == 0
assert names(checked.stdout) == original, 'The original complete native inventory changed'
full_inventory = args.output / 'complete.inventory.log'
full_inventory.write_text(checked.stdout + checked.stderr)
groups = [[] for _ in args.cpus]
loads = [0 for _ in args.cpus]


def weight(name):
    # Scheduling estimates are never used to select, drop or retry a test.
    if name.startswith('frontend_test262_subset::'):
        return 800
    if name.startswith('intl::'):
        return 25
    if name.startswith('frontend::') or name.startswith('known_failures::'):
        return 1
    return 3


for name in sorted(original, key=lambda item: (-weight(item), item)):
    index = min(range(len(groups)), key=lambda group: (loads[group], group))
    groups[index].append(name)
    loads[index] += weight(name)
assert sorted(name for group in groups for name in group) == sorted(original)
assert all(groups)
executable_sha = digest(args.executable)
receipt = {
    'policy': 'Complete logical target: every original qualified ID exactly once in disjoint native libtest processes, each on one inherited CPU and one test thread; original bodies and deadlines unchanged; zero ignored required.',
    'driver_sha256': digest(Path(__file__)),
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
    label = args.output.name + '-partition-' + str(row['index'])
    log = args.output / f'partition-{row["index"]}.launcher.log'
    stream = log.open('w')
    launch = ['taskset', '-c', str(row['cpu']), './scripts/run-watched.sh',
              '--label', label, '--stall', '3600', '--poll', '1', '--',
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
