#!/usr/bin/env python3
"""Describe repeatability in a checked, case-wise ABBA performance receipt.

This consumes preserved observations; it runs no benchmark and sets no automatic
performance threshold. Two round medians per source establish an observed range,
not a confidence interval or a bound on future noise.
"""
import argparse
import hashlib
import json
from pathlib import Path
from statistics import median


def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def require(condition, message):
    if not condition:
        raise ValueError(message)


def positive_samples(values, count):
    return (isinstance(values, list) and len(values) == count
            and all(type(value) is int and value > 0 for value in values))


def analyze(path):
    report = json.loads(path.read_text())
    require(report['schema'] in {'scriptw-frozen-string-hash-diagnostic-v1',
                                 'scriptw-frozen-paired-diagnostic-v1'},
            'unsupported paired receipt schema')
    require(report['complete'] is True, 'incomplete paired receipt')
    samples = report['configuration']['samples']
    warmups = report['configuration']['warmups']
    require(type(samples) is int and 3 <= samples <= 100
            and type(warmups) is int and 1 <= warmups <= 100, 'invalid sample plan')
    require(set(report['identities']) == set(report['runners']) == {'before', 'after'},
            'expected exactly two identified products')
    rows = report['observations']
    require(rows and len(rows) % 4 == 0, 'incomplete ABBA case inventory')
    cases, seen, raw_files = [], set(), set()
    for offset in range(0, len(rows), 4):
        group = rows[offset:offset + 4]
        name = group[0]['file']
        require(name not in seen and all(row['file'] == name for row in group),
                'cases must be contiguous and unique')
        seen.add(name)
        require([row['side'] for row in group] == ['before', 'after', 'after', 'before'],
                'expected case-wise ABBA observation order')
        source_hashes, outputs, runtime_policies = set(), set(), set()
        per_source = {}
        for index, row in enumerate(group):
            side = row['side']
            require(row['label'] == f'{name}-{index}-{side}', 'round label differs')
            require(row['process']['exit_code'] == 0 and row['process']['timed_out'] is False,
                    'failed or timed-out benchmark process')
            require(positive_samples(row['execution_ns'], samples), 'invalid elapsed samples')
            require(row['median_ns'] == median(row['execution_ns']), 'stored median differs')
            raw_name = row['raw']
            require(Path(raw_name).name == raw_name and raw_name not in raw_files,
                    'raw observations must be unique sibling files')
            raw_files.add(raw_name)
            raw_path = path.parent / raw_name
            require(sha(raw_path) == row['raw_sha256'], 'raw observation checksum differs')
            raw = json.loads(raw_path.read_text())
            require(raw['schema'] == 'scriptw-wasmtime-baseline-v2'
                    and raw['samples'] == samples and raw['warmups'] == warmups,
                    'raw sample plan differs')
            require(len(raw['cases']) == 1, 'expected one case per child')
            case = raw['cases'][0]
            identity = report['identities'][side]
            require(row['source_commit'] == raw['source_commit']
                    == identity['source_revision']['commit'], 'source revision differs')
            require(row['source_fingerprint'] == raw['source_fingerprint']
                    == identity['source_fingerprint'], 'source fingerprint differs')
            require(row['runner_sha256'] == raw['executable_sha256']
                    == report['runners'][side]['sha256'], 'runner identity differs')
            require(case['file'] == name and case['source_sha256'] == row['source_sha256']
                    and case['wasm_sha256'] == row['wasm_sha256'], 'case identity differs')
            observations = case['observations']
            require(len(observations) == warmups + samples
                    and [item['warmup'] for item in observations]
                    == [True] * warmups + [False] * samples, 'warmup ordering differs')
            measured = observations[warmups:]
            require([item['execution_ns'] for item in measured] == row['execution_ns']
                    and all(item['module_cache'] == 'Hit' for item in measured),
                    'raw execution samples or module cache differ')
            source_hashes.add(case['source_sha256'])
            outputs.update(item['stdout'] for item in observations)
            runtime_policies.update(item['runtime_policy'] for item in observations)
            binding = dict(source_commit=raw['source_commit'],
                           source_fingerprint=raw['source_fingerprint'],
                           runner_sha256=raw['executable_sha256'],
                           wasm_sha256=case['wasm_sha256'], linked_runtime=case['linked_runtime'])
            cohort = per_source.setdefault(side, dict(identity=binding, rounds=[]))
            require(cohort['identity'] == binding, 'same-source rounds changed product or Wasm')
            cohort['rounds'].append(dict(index=index, raw=raw_name, raw_sha256=row['raw_sha256'],
                                        samples_ns=row['execution_ns'], median_ns=row['median_ns']))
        require(len(source_hashes) == len(outputs) == len(runtime_policies) == 1,
                'fixture, stdout or runtime policy differs between observations')
        for cohort in per_source.values():
            medians = [row['median_ns'] for row in cohort['rounds']]
            values = [value for row in cohort['rounds'] for value in row['samples_ns']]
            cohort.update(round_medians_ns=medians, median_of_round_medians_ns=median(medians),
                          round_median_range_ns=[min(medians), max(medians)],
                          round_median_spread_percent=(max(medians) / min(medians) - 1) * 100,
                          sample_range_ns=[min(values), max(values)])
        effects = [(group[1]['median_ns'] / group[0]['median_ns'] - 1) * 100,
                   (group[2]['median_ns'] / group[3]['median_ns'] - 1) * 100]
        cases.append(dict(file=name, source_sha256=source_hashes.pop(), stdout=outputs.pop(),
                          runtime_policy=runtime_policies.pop(), cohorts=per_source,
                          paired_effect_percent=effects,
                          paired_effect_range_percent=[min(effects), max(effects)],
                          median_ratio_effect_percent=(
                              per_source['after']['median_of_round_medians_ns'] /
                              per_source['before']['median_of_round_medians_ns'] - 1) * 100))
    return dict(schema='scriptw-paired-repeatability-v1', receipt=path.name,
                receipt_sha256=sha(path), analyzer_sha256=sha(Path(__file__)),
                configuration=report['configuration'], cpu=report['cpu'],
                measurement_harness_sha256=report['harness_sha256'], cases=cases,
                interpretation=[
                    'Historical execution/completion observations; no new benchmark was run.',
                    'Output consistency is checked across rounds; source/oracle correctness relies on the identified original producer evidence.',
                    'Each source has two process round medians; ranges include timing drift and other observed variation.',
                    'Spread is (larger/smaller - 1) * 100. Paired effects compare adjacent A/B and B/A rounds.',
                    'Negative effects mean faster after-source execution; all raw samples and slower rounds remain.',
                    'Observed ranges are not confidence intervals, future-noise bounds or an automatic acceptance threshold.',
                    'Use these ranges to choose follow-up controls before accepting small future gains; they do not retroactively validate decisions.'
                ])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('receipt', type=Path)
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    try:
        result = analyze(args.receipt)
        with args.output.open('x') as stream:
            stream.write(json.dumps(result, indent=2) + '\n')
    except (ValueError, KeyError, TypeError, OSError) as error:
        parser.exit(1, f'paired-repeatability: {error}\n')
    print(f"Analyzed {len(result['cases'])} preserved ABBA cases: {args.output}")


if __name__ == '__main__':
    main()
