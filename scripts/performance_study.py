#!/usr/bin/env python3
"""Attribute checked benchmark reports without changing their timing boundaries."""
import argparse
import json
from pathlib import Path
import statistics

from benchmark_metrics import digest, load_report


def study(report_paths):
    reports = []
    scales = []
    for path in report_paths:
        report = load_report(path)
        cases = []
        plans = {case['file']: case for case in report['workloads']}
        for case in report['execution']['wasmtime']['cases']:
            observations = [row for row in case['observations'] if not row['warmup']]
            phases = {name: statistics.median(row['phases_ns'][name] for row in observations)
                      for name in observations[0]['phases_ns']}
            execution = statistics.median(row['execution_ns'] for row in observations)
            unassigned = statistics.median(row['phases_ns']['total'] - row['execution_ns'] - sum(
                value for name, value in row['phases_ns'].items() if name != 'total') for row in observations)
            plan = plans[case['file']]
            parameters = plan.get('parameters', {})
            entry = dict(file=case['file'], parameters=parameters, execution_ns=execution,
                         phases_ns=phases, unassigned_total_ns=unassigned,
                         execution_fraction_of_total=execution / phases['total'] if phases['total'] else None,
                         source_compilation_profile=case.get('source_compilation_profile', 'unrecorded'),
                         linked_runtime=case.get('linked_runtime'), compilation_ns=case['compilation_ns'],
                         process_memory_boundary='whole process; not live JS heap bytes')
            cases.append(entry)
            if all(key in parameters for key in ('rows', 'width', 'read_passes')):
                scales.append(dict(report_sha256=digest(path), **entry))
        reports.append(dict(path=str(Path(path).resolve()), sha256=digest(path),
                            source_commit=report['source_commit'], samples=report['samples'],
                            warmups=report['configuration']['warmups'], cases=cases))
    return dict(schema='scriptw-performance-study-v1', reports=reports, row_scaling=scales,
                interpretation='Raw whole-invocation medians include construction. Scaling gives attribution evidence, not a separately timed traversal span. Phase fractions use medians and need not add to one. No gain or statistical-significance verdict.')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('reports', type=Path, nargs='+')
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    result = study(args.reports)
    # Evidence is never overwritten; retain failed and superseded investigations.
    with args.output.open('x') as output:
        output.write(json.dumps(result, indent=2) + '\n')
    print(f"Attributed {sum(len(r['cases']) for r in result['reports'])} cases; retained {args.output}")


if __name__ == '__main__':
    main()
