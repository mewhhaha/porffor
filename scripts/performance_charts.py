"""Deterministic SVG comparisons from admitted Scriptw/V8 measurements.

The publisher owns evidence admission. No plotting dependency, browser, network
access or timing run is required to render the latest checkpoint.
"""
from datetime import datetime, timezone
from html import escape
import math


METRICS = (
    ('cold', 'Cold start', 'Startup'),
    ('hot', 'Cached start', 'Startup'),
    ('integer', '10,000 integer steps', 'Execution'),
    ('objects', '500 objects', 'Execution'),
    ('retained', '8,192 retained objects', 'Execution'),
)
WIDTH = 1040
INK = '#24292f'
MUTED = '#57606a'
GRID = '#d8dee4'
V8 = '#d97706'
SCRIPTW = '#0969da'


def _text(x, y, value, *, size=13, fill=INK, weight=None, anchor=None):
    attributes = f'x="{x:.3f}" y="{y:.3f}" font-size="{size}" fill="{fill}"'
    if weight:
        attributes += f' font-weight="{weight}"'
    if anchor:
        attributes += f' text-anchor="{anchor}"'
    return f'<text {attributes}>{escape(str(value))}</text>'


def _line(x1, y1, x2, y2, *, stroke=GRID):
    return (f'<line x1="{x1:.3f}" y1="{y1:.3f}" x2="{x2:.3f}" '
            f'y2="{y2:.3f}" stroke="{stroke}"/>')


def _svg(title, description, height, content):
    return '\n'.join([
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {WIDTH} {height}" '
        'role="img" aria-labelledby="title description">',
        f'<title id="title">{escape(title)}</title>',
        f'<desc id="description">{escape(description)}</desc>',
        f'<rect width="{WIDTH}" height="{height}" rx="12" fill="#ffffff"/>',
        '<g font-family="system-ui, -apple-system, BlinkMacSystemFont, Segoe UI, sans-serif">',
        *content, '</g>', '</svg>', '',
    ])


def _timestamp(value):
    instant = datetime.fromisoformat(value.replace('Z', '+00:00'))
    if instant.utcoffset() is None:
        raise ValueError('performance measurement timestamps need a timezone')
    return instant.astimezone(timezone.utc)


def _checked(point):
    metrics = point['metrics']
    if len(metrics) != len(METRICS) or {m['id'] for m in metrics} != {m[0] for m in METRICS}:
        raise ValueError('performance charts need exactly the five README metrics')
    rows = {metric['id']: metric for metric in metrics}
    for key, _, group in METRICS:
        row = rows[key]
        if row['group'] != group:
            raise ValueError('performance metric has the wrong timing group')
        for fields, role in ((('scriptw_ns', 'v8_ns'), 'timings'),
                             (('scriptw_rss_bytes', 'v8_rss_bytes'), 'RSS values')):
            values = [row[field] for field in fields]
            if any(type(value) not in (int, float) or not math.isfinite(value) or value <= 0
                   for value in values):
                raise ValueError(f'chart {role} must be finite and positive')
            ratio = values[0] / values[1]
            if not math.isfinite(ratio) or ratio <= 0:
                raise ValueError(f'chart {role} ratio must be finite and positive')
    _timestamp(point['measured_at'])
    return rows


def _ratio(value):
    if value < .01:
        return f'{value:.3g}×'
    if value >= 1000:
        return f'{value:,.1f}'.rstrip('0').rstrip('.') + '×'
    return f'{value:.2f}'.rstrip('0').rstrip('.') + '×'


def _milliseconds(nanoseconds):
    value = nanoseconds / 1_000_000
    return f'{value:.3f}'.rstrip('0').rstrip('.') if nanoseconds >= 1000 else f'{value:.3g}'


def _heading(title, subtitle):
    content = [_text(28, 35, title, size=22, weight=600),
               _text(28, 61, subtitle, size=12, fill=MUTED)]
    for x, label, color in ((643, 'Node / V8', V8), (801, 'Scriptw / Wasmtime', SCRIPTW)):
        content += [f'<rect x="{x}" y="23" width="14" height="14" rx="2" fill="{color}"/>',
                    _text(x + 22, 35, label, size=13)]
    return content


def _paired_bars(row, y, fields, maximum, format_value):
    """Both bars share a zero origin and a linear scale, with no minimum width."""
    left, width = 268, 502
    content = [_line(left, y - 3, left, y + 38)]
    for engine, field, color, offset in (('V8', fields[1], V8, 0),
                                          ('Scriptw', fields[0], SCRIPTW, 21)):
        value = row[field]
        bar_width = value / maximum * width
        label = format_value(value)
        content += [
            f'<rect x="{left}" y="{y + offset}" width="{bar_width:.6f}" height="14" '
            f'fill="{color}" data-engine="{engine}" data-metric="{escape(row["id"], quote=True)}">'
            f'<title>{escape(row["label"])} · {engine}: {escape(label)}</title></rect>',
            _text(left + bar_width + 9, y + offset + 12, label, size=12, fill=color, weight=500),
        ]
    return content


def _resources(machine):
    cpus = len(machine.get('measurement_affinity', []))
    memory = machine.get('effective_memory_bytes')
    parts = [f'{cpus} measured CPU' + ('s' if cpus != 1 else '')] if cpus else []
    if memory:
        parts.append(f'{memory / 2**30:g} GiB memory limit')
    if any(row.get('memory.swap.max') == '0' for row in machine.get('cgroups', [])):
        parts.append('no swap')
    return ' · '.join([machine['cpu_model'], *parts])


def render_current(point):
    """Compare absolute median times with two zero-based bars per workload."""
    rows = _checked(point)
    measured = _timestamp(point['measured_at']).strftime('%Y-%m-%d %H:%M UTC')
    description = ('Median elapsed time in milliseconds; lower is better. Each workload has '
                   'its own zero-based linear scale, shared by its V8 and Scriptw bars. '
                   'Ratios are Scriptw divided by V8. ' + '; '.join(
        f'{rows[key]["label"]}: V8 {_milliseconds(rows[key]["v8_ns"])} ms, '
        f'Scriptw {_milliseconds(rows[key]["scriptw_ns"])} ms, '
        f'{_ratio(rows[key]["scriptw_ns"] / rows[key]["v8_ns"])}'
        for key, *_ in METRICS))
    content = _heading('Elapsed time', 'Median · ms · lower is better · independent linear scale per workload')
    content += [
        _text(28, 84, f'{measured} · compiler {point["source_commit"][:9]} · '
              f'{point["samples"]} samples · {point["warmups"]} execution warmups', size=11, fill=MUTED),
        _text(28, 117, 'STARTUP', size=11, weight=600, fill=MUTED),
        _text(1012, 117, 'Scriptw / V8', size=11, fill=MUTED, anchor='end'),
        _text(28, 277, 'EXECUTION', size=11, weight=600, fill=MUTED),
        _line(28, 251, 1012, 251),
    ]
    for (key, _, _), y in zip(METRICS, (133, 195, 293, 355, 417), strict=True):
        row = rows[key]
        content.append(_text(28, y + 24, row['label'], size=14, weight=500))
        content += _paired_bars(row, y, ('scriptw_ns', 'v8_ns'),
                                max(row['scriptw_ns'], row['v8_ns']),
                                lambda value: _milliseconds(value) + ' ms')
        content.append(_text(1012, y + 25, _ratio(row['scriptw_ns'] / row['v8_ns']),
                             size=19, weight=600, anchor='end'))
    content += [
        _text(28, 483, 'Startup: fresh process · Execution: fresh Store / V8 context', size=11, fill=MUTED),
        _text(28, 505, _resources(point['machine']), size=11, fill=MUTED),
        _text(28, 524, f'Wasmtime {point["wasmtime_version"]} · Node {point["node_version"]} / '
              f'V8 {point["v8_version"]}', size=11, fill=MUTED),
    ]
    return _svg('Scriptw versus V8 elapsed time', description, 545, content)


def render_memory(point):
    """Show startup medians and the whole retained-object child RSS separately."""
    rows = _checked(point)
    keys = ('cold', 'hot', 'retained')
    fields = ('scriptw_rss_bytes', 'v8_rss_bytes')
    maximum = max(rows[key][field] for key in keys for field in fields)

    def mib(value):
        return f'{value / 2**20:.2f} MiB'

    description = ('Peak process RSS in MiB, including runtime and host overhead; lower is better. '
                   'Shared zero-based linear scale. Startup values are medians; retained-object '
                   'values cover the whole child process, including compilation and warmups. '
                   'This is not JavaScript heap usage. ' + '; '.join(
        f'{rows[key]["label"]}: V8 {mib(rows[key][fields[1]])}, Scriptw {mib(rows[key][fields[0]])}'
        for key in keys))
    content = _heading('Peak process memory', 'RSS · MiB · lower is better · shared linear scale')
    for key, y, note in (('cold', 92, 'median'), ('hot', 164, 'median'),
                          ('retained', 236, 'whole child process')):
        row = rows[key]
        content += [_text(28, y + 14, row['label'], size=14, weight=500),
                    _text(28, y + 32, note, size=11, fill=MUTED)]
        content += _paired_bars(row, y, fields, maximum, mib)
    content.append(_text(28, 311, 'Includes host/runtime overhead; retained-object run includes compilation and warmups.',
                         size=11, fill=MUTED))
    return _svg('Scriptw versus V8 peak process memory', description, 332, content)
