#!/usr/bin/env python3
"""Analyze the preserved first-wave row sweep without rerunning benchmarks.

The optional plot requires matplotlib. All timings include construction;
normalizing by row count does not turn them into isolated traversal timings.
"""
import argparse
import base64
import gzip
import hashlib
import json
from pathlib import Path
import re
from statistics import median


def digest(data):
    return hashlib.sha256(data).hexdigest()


def require(condition, message):
    if not condition:
        raise ValueError(message)


def analyze(path):
    raw = path.read_bytes()
    evidence = json.loads(raw)
    assets = {item['path']: item for item in evidence['assets']}
    require(len(assets) == len(evidence['assets']), 'duplicate evidence paths')

    def asset(name):
        entry = assets[name]
        require(entry['encoding'] == 'gzip+base64', f'{name}: unexpected encoding')
        compressed = base64.b64decode(entry['data'], validate=True)
        require(digest(compressed) == entry['gzip_sha256'], f'{name}: gzip digest mismatch')
        data = gzip.decompress(compressed)
        require(len(data) == entry['bytes'] and digest(data) == entry['sha256'],
                f'{name}: payload mismatch')
        return data

    cohorts, rows = {}, []
    for name in ['before-screen', 'before-supplemental', 'current-screen']:
        report_name = f'runs/{name}/report.json'
        report_raw = asset(report_name)
        report = json.loads(report_raw)
        advertised = evidence['reports'][name]
        require(report['complete'] and digest(report_raw) == advertised['sha256'],
                f'{name}: incomplete or mismatched report')
        require(report['source_commit'] == advertised['source'], f'{name}: source mismatch')
        require(report['summaries'] == advertised['summaries'], f'{name}: summary mismatch')
        cohorts[name] = {key: report[key] for key in [
            'source_commit', 'cli_sha256', 'runtime_runner_sha256', 'compiler_identity',
            'samples', 'configuration', 'machine', 'started_at', 'finished_at',
        ]}
        cohorts[name]['report_sha256'] = digest(report_raw)
        workloads = {item['file']: item for item in report['workloads']}
        for case in report['execution']['wasmtime']['cases']:
            match = re.fullmatch(r'rows-n(\d+)-w(\d+)-p(\d+)\.js', case['file'])
            if not match:
                continue
            count, width, passes = map(int, match.groups())
            name_in_report = case['file']
            workload = workloads[name_in_report]
            source = asset(f'runs/{name}/workload/{name_in_report}')
            require(digest(source) == workload['sha256'] == case['source_sha256'],
                    f'{name}/{name_in_report}: source mismatch')
            expected = (2 * count + width - 2 + passes * (
                width * count * (count - 1) // 2 + count * width * (width - 1) // 2))
            require(workload['stdout'] == f'{expected}\n', 'row checksum mismatch')
            node_case, = [item for item in report['execution']['v8']['cases']
                          if item['file'] == name_in_report]
            require(node_case['source_sha256'] == case['source_sha256'], 'V8 source mismatch')
            runtimes = {}
            for runtime, measured in [('wasmtime', case), ('v8', node_case)]:
                observations = measured['observations']
                require(all(item['stdout'] == workload['stdout'] for item in observations),
                        f'{name}/{name_in_report}/{runtime}: output mismatch')
                samples = [item for item in observations if not item['warmup']]
                require(len(samples) == report['samples'], 'sample count mismatch')
                require(len(observations) - len(samples) == report['configuration']['warmups'],
                        'warmup count mismatch')
                timings = [item['execution_ns'] for item in samples]
                summary = report['summaries']['execution'][runtime][name_in_report]['execution_ns']
                require(timings == summary['samples'] and median(timings) == summary['median'],
                        'execution summary mismatch')
                entry = dict(samples_ns=timings, median_ns=median(timings),
                             minimum_ns=min(timings), maximum_ns=max(timings),
                             median_ns_per_row=median(timings) / count,
                             median_ns_per_retained_value=median(timings) / (count * width))
                if runtime == 'wasmtime':
                    require(all(item['module_cache'] == 'Hit' for item in samples),
                            'measured module-cache miss')
                    entry['gc_capacity_before_bytes'] = sorted({
                        item['memory']['gc_capacity_before_bytes'] for item in samples})
                    entry['gc_capacity_after_bytes'] = sorted({
                        item['memory']['gc_capacity_after_bytes'] for item in samples})
                runtimes[runtime] = entry
            rows.append(dict(cohort=name, file=name_in_report, rows=count, width=width,
                             read_passes=passes, source_sha256=digest(source),
                             stdout=workload['stdout'], runtimes=runtimes))

    require(len(rows) == 21, 'expected two nine-case row sweeps and three width controls')
    baseline = cohorts['before-screen']
    supplemental = cohorts['before-supplemental']
    for key in ['source_commit', 'cli_sha256', 'runtime_runner_sha256', 'machine']:
        require(baseline[key] == supplemental[key], f'width controls change {key}')
    require({key: value for key, value in baseline['configuration'].items() if key != 'warmups'}
            == {key: value for key, value in supplemental['configuration'].items() if key != 'warmups'},
            'width controls change measurement policy beyond their separate warmup count')
    growth = []
    for cohort in ['before-screen', 'current-screen']:
        for passes in [0, 1, 4]:
            points = sorted((row for row in rows if row['cohort'] == cohort
                             and row['width'] == 4 and row['read_passes'] == passes),
                            key=lambda row: row['rows'])
            require([row['rows'] for row in points] == [256, 1024, 4096], 'missing row count')
            for first, last in zip(points, points[1:]):
                ratio = last['runtimes']['wasmtime']['median_ns'] / first['runtimes']['wasmtime']['median_ns']
                row_ratio = last['rows'] / first['rows']
                growth.append(dict(cohort=cohort, read_passes=passes,
                                   from_rows=first['rows'], to_rows=last['rows'],
                                   row_count_ratio=row_ratio, elapsed_median_ratio=ratio,
                                   normalized_per_row_ratio=ratio / row_ratio))
    return dict(schema='scriptw-preserved-row-scaling-v1', evidence_file=path.name,
                evidence_sha256=digest(raw),
                generator=dict(file=Path(__file__).name, sha256=digest(Path(__file__).read_bytes())),
                cohorts=cohorts, rows=rows, growth=growth,
                limitations=[
                    'Historical source cohorts; these are not current-product measurements.',
                    'Each panel retains its cohort sample/warmup counts. Min/max bars are observed spread, not confidence intervals.',
                    'Every timing includes row construction, reads, checksum and completion.',
                    'Zero-pass rows include construction and the final length/element checksum.',
                    'Per-value normalization divides the whole invocation by retained array elements (rows times width), not by the number of read operations.',
                    'Capacity snapshots are not live bytes, allocation counts, peak heap or proof of GC causation.',
                    'The width controls share the original source and machine but use a separate sample/warmup cohort; they do not cover the later source.',
                    'No timings are subtracted, no sample is excluded, and no optimization speedup is inferred from these plots.',
                ])


def plot(report, path):
    import matplotlib
    matplotlib.use('Agg')
    import matplotlib.pyplot as plt
    matplotlib.rcParams['svg.hashsalt'] = 'scriptw-preserved-row-scaling-v1'
    matplotlib.rcParams['font.family'] = 'DejaVu Sans'
    fig, axes = plt.subplots(1, 3, figsize=(14, 4.6), layout='constrained')
    colors = {0: '#215c9e', 1: '#c46b13', 4: '#19805c'}
    for axis, cohort in zip(axes[:2], ['before-screen', 'current-screen']):
        for passes in [0, 1, 4]:
            points = sorted((row for row in report['rows'] if row['cohort'] == cohort
                             and row['read_passes'] == passes), key=lambda row: row['rows'])
            x = [row['rows'] for row in points]
            y = [row['runtimes']['wasmtime']['median_ns_per_row'] / 1000 for row in points]
            low = [(row['runtimes']['wasmtime']['median_ns'] - row['runtimes']['wasmtime']['minimum_ns'])
                   / row['rows'] / 1000 for row in points]
            high = [(row['runtimes']['wasmtime']['maximum_ns'] - row['runtimes']['wasmtime']['median_ns'])
                    / row['rows'] / 1000 for row in points]
            axis.errorbar(x, y, yerr=[low, high], marker='o', capsize=4,
                          color=colors[passes], label=f'{passes} read passes')
        axis.set_xscale('log', base=4)
        axis.set_xticks([256, 1024, 4096], ['256', '1,024', '4,096'])
        axis.set_xlabel('Retained rows (array width 4)')
        axis.set_ylabel('Whole-invocation µs / row')
        cohort_info = report['cohorts'][cohort]
        axis.set_title(f"{cohort_info['source_commit'][:9]}\n{cohort_info['samples']} samples / "
                       f"{cohort_info['configuration']['warmups']} warmups")
        capacities = []
        for count in [256, 1024, 4096]:
            observed = sorted({capacity / 2**20 for row in report['rows']
                               if row['cohort'] == cohort and row['rows'] == count
                               for capacity in row['runtimes']['wasmtime']['gc_capacity_after_bytes']})
            capacities.append(','.join(f'{value:g}' for value in observed))
        axis.text(.03, .96, 'Capacity after: ' + ' / '.join(capacities) + ' MiB', transform=axis.transAxes,
                  va='top', fontsize=9)
        axis.legend(loc='center left', fontsize=8)
        axis.grid(alpha=.2)
        axis.set_ylim(bottom=0)
    points = sorted((row for row in report['rows'] if row['cohort'] == 'before-supplemental'),
                    key=lambda row: row['width'])
    axis = axes[2]
    for runtime, color in [('wasmtime', '#215c9e'), ('v8', '#8b458e')]:
        x = [row['width'] for row in points]
        y = [row['runtimes'][runtime]['median_ns_per_retained_value'] / 1000 for row in points]
        low = [(row['runtimes'][runtime]['median_ns'] - row['runtimes'][runtime]['minimum_ns'])
               / (row['rows'] * row['width']) / 1000 for row in points]
        high = [(row['runtimes'][runtime]['maximum_ns'] - row['runtimes'][runtime]['median_ns'])
                / (row['rows'] * row['width']) / 1000 for row in points]
        axis.errorbar(x, y, yerr=[low, high], marker='o', capsize=4, color=color, label=runtime)
    axis.set_xticks([2, 4, 8])
    axis.set_xlabel('Values per row (1,024 rows, one pass)')
    axis.set_ylabel('Whole-invocation µs / retained array element')
    cohort_info = report['cohorts']['before-supplemental']
    axis.set_title(f"Original-source width controls\n{cohort_info['samples']} samples / "
                   f"{cohort_info['configuration']['warmups']} warmups")
    axis.legend()
    axis.grid(alpha=.2)
    axis.set_ylim(bottom=0)
    fig.suptitle('Historical row scaling: medians and observed min/max, not isolated traversal', fontsize=12)
    fig.savefig(path, metadata={'Date': None} if path.suffix == '.svg' else None)
    plt.close(fig)
    return matplotlib.__version__


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--evidence', type=Path,
                        default=Path('docs/rust-rewrite/performance-first-wave-20261022.json'))
    parser.add_argument('--output-dir', type=Path, required=True)
    parser.add_argument('--plot', action='store_true', help='also write an SVG using matplotlib')
    options = parser.parse_args()
    report = analyze(options.evidence)
    options.output_dir.mkdir(parents=True, exist_ok=True)
    if options.plot:
        image = options.output_dir / 'row-scaling.svg'
        plot_version = plot(report, image)
        report['plot'] = dict(path=image.name, sha256=digest(image.read_bytes()),
                              matplotlib_version=plot_version)
    output = options.output_dir / 'row-scaling.json'
    output.write_text(json.dumps(report, indent=2) + '\n')
    print(f'Validated {len(report["rows"])} preserved cases; wrote {output}')


if __name__ == '__main__':
    main()
