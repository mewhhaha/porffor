"""Deterministic SVGs for admitted Scriptw/V8 performance checkpoints.

The publisher owns evidence admission. These renderers consume its normalized
points and keep incompatible measurement cohorts out of a connected history.
No plotting dependency, browser, network access or timing run is required.
"""
from datetime import datetime, timezone
from html import escape
import math


METRICS = (
    ('cold', 'Cold start', 'Startup', '#0969da', None),
    ('hot', 'Cached start', 'Startup', '#8250df', '7 4'),
    ('integer', '10,000 integer steps', 'Execution', '#1a7f37', None),
    ('objects', '500 objects', 'Execution', '#bc4c00', '7 4'),
    ('retained', '8,192 retained objects', 'Execution', '#cf222e', '2 4'),
)
WIDTH = 1040
INK = '#24292f'
MUTED = '#57606a'
GRID = '#d8dee4'


def _text(x, y, value, *, size=13, fill=INK, weight=None, anchor=None):
    attributes = f'x="{x:.3f}" y="{y:.3f}" font-size="{size}" fill="{fill}"'
    if weight:
        attributes += f' font-weight="{weight}"'
    if anchor:
        attributes += f' text-anchor="{anchor}"'
    return f'<text {attributes}>{escape(str(value))}</text>'


def _line(x1, y1, x2, y2, *, stroke=GRID, width=1, dash=None):
    attributes = (f'x1="{x1:.3f}" y1="{y1:.3f}" x2="{x2:.3f}" '
                  f'y2="{y2:.3f}" stroke="{stroke}" stroke-width="{width}"')
    if dash:
        attributes += f' stroke-dasharray="{dash}"'
    return f'<line {attributes}/>'


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
    for key, _, group, _, _ in METRICS:
        row = rows[key]
        if row['group'] != group:
            raise ValueError('performance metric has the wrong timing group')
        values = (row['scriptw_ns'], row['v8_ns'])
        if any(type(value) not in (int, float) or not math.isfinite(value) or value <= 0
               for value in values):
            raise ValueError('chart timings must be finite and positive')
        ratio = row['scriptw_ns'] / row['v8_ns']
        if not math.isfinite(ratio) or ratio <= 0:
            raise ValueError('chart timing ratio must be finite and positive')
    _timestamp(point['measured_at'])
    return rows


def _ratio(value):
    if value < .01:
        return f'{value:.3g}×'
    if value >= 1000:
        return f'{value:,.1f}'.rstrip('0').rstrip('.') + '×'
    return f'{value:.2f}'.rstrip('0').rstrip('.') + '×'


def _milliseconds(nanoseconds):
    return f'{nanoseconds / 1_000_000:.6g}'


def _scale(ratios):
    """Log exponents include V8=1 with padding and at most seven tick marks."""
    lower = min(0, math.floor(math.log10(min(ratios)))) - .15
    upper = max(1, math.ceil(math.log10(max(ratios)))) + .08
    first, last = math.ceil(lower), math.floor(upper)
    step = max(1, math.ceil((last - first) / 5))
    ticks = sorted({0, *range(first, last + 1, step)})
    return lower, upper, ticks


def _tick_label(exponent):
    if -3 <= exponent <= 3:
        return _ratio(10 ** exponent)
    return f'1e{exponent:+}×'


def _context(point):
    return (f"{point['machine']['cpu_model']} · Node {point['node_version']} / "
            f"V8 {point['v8_version']} · Wasmtime {point['wasmtime_version']}")


def render_current(point):
    """Show the five exact median-time ratios against a visible V8 reference."""
    rows = _checked(point)
    ratios = {key: rows[key]['scriptw_ns'] / rows[key]['v8_ns'] for key, *_ in METRICS}
    lower, upper, ticks = _scale(list(ratios.values()))
    left, right, top, bottom = 278, 778, 112, 334

    def position(value):
        return left + (math.log10(value) - lower) / (upper - lower) * (right - left)

    measured = _timestamp(point['measured_at']).strftime('%Y-%m-%d %H:%M UTC')
    description = ('Scriptw elapsed time divided by V8 elapsed time. V8 equals one; '
                   'lower is better. Logarithmic horizontal scale. ' + '; '.join(
        f'{rows[key]["label"]}: Scriptw {_milliseconds(rows[key]["scriptw_ns"])} ms, '
        f'V8 {_milliseconds(rows[key]["v8_ns"])} ms, {_ratio(ratios[key])}'
        for key, *_ in METRICS))
    content = [
        _text(28, 34, 'Scriptw performance vs V8', size=22, weight=600),
        _text(28, 57, f'{measured} · compiler {point["source_commit"][:9]} · '
              f'{point["samples"]} samples · {point["warmups"]} execution warmups', size=12, fill=MUTED),
        _text(28, 93, 'Measurement', size=12, weight=600),
        _text(left, 93, 'Elapsed-time ratio · lower is better · log scale', size=12, fill=MUTED),
        _text(1012, 93, 'Scriptw / V8', size=12, weight=600, anchor='end'),
    ]
    reference = position(1)
    for exponent in ticks:
        x = left + (exponent - lower) / (upper - lower) * (right - left)
        content += [_line(x, top, x, bottom),
                    _text(x, 355, _tick_label(exponent), size=11, fill=MUTED, anchor='middle')]
    content.append(_line(reference, top, reference, bottom, stroke=INK, width=1.5, dash='4 4'))
    content.append(_text(reference + 7, 110, 'V8 = 1×', size=10, fill=MUTED))
    for index, (key, _, _, color, _) in enumerate(METRICS):
        y = 135 + index * 44
        row, ratio = rows[key], ratios[key]
        target = position(ratio)
        content.append(_text(28, y + 1, row['label'], size=14, weight=500))
        content.append(_line(reference, y - 4, target, y - 4, stroke=color, width=13))
        content.append(f'<circle cx="{target:.3f}" cy="{y - 4:.3f}" r="6.5" fill="{color}">'
                       f'<title>{escape(row["label"])}: {_ratio(ratio)}</title></circle>')
        content.append(_text(1012, y - 2, _ratio(ratio), size=19, weight=600, anchor='end'))
        content.append(_text(1012, y + 15,
                             f'{_milliseconds(row["scriptw_ns"])} / {_milliseconds(row["v8_ns"])} ms',
                             size=11, fill=MUTED, anchor='end'))
    content += [
        _text(28, 383, 'Execution uses a fresh Store / V8 context per invocation; startup uses fresh processes.',
              size=11, fill=MUTED),
        _text(28, 403, 'Measured medians, not confidence intervals. Full samples, scope and provenance are retained in the history.',
              size=11, fill=MUTED),
    ]
    return _svg('Scriptw performance versus V8', description, 423, content)


def _legend(content, metrics, x, y):
    for key, label, _, color, dash in metrics:
        width = 155 if key in ('cold', 'hot') else 215
        content.append(_line(x, y - 4, x + 25, y - 4, stroke=color, width=2.5, dash=dash))
        content.append(_text(x + 32, y, label, size=12))
        x += width


def render_history(points):
    """Plot matched checkpoints with ordinal x spacing; retain no mixed line."""
    if not points:
        raise ValueError('performance history needs a measured checkpoint')
    if len({point['cohort_id'] for point in points}) != 1:
        raise ValueError('cannot connect different performance cohorts')
    instants = [_timestamp(point['measured_at']) for point in points]
    if instants != sorted(instants):
        raise ValueError('performance history must be in measurement order')
    all_rows = [_checked(point) for point in points]
    total = len(points)
    points, all_rows = points[-12:], all_rows[-12:]
    latest = points[-1]
    content = [
        _text(28, 34, 'Performance history vs V8', size=22, weight=600),
        _text(28, 57, 'Scriptw / V8 elapsed time · lower is better · V8 = 1× · logarithmic vertical scales',
              size=12, fill=MUTED),
    ]
    if len(points) == 1:
        content.append(_text(28, 79, 'One measured checkpoint; no trend is drawn.', size=12, fill=MUTED))
    else:
        content.append(_text(28, 79, f'{len(points)} of {total} matched checkpoints shown · '
                             'equal checkpoint spacing, not elapsed calendar time', size=12, fill=MUTED))
    left, right = 88, 985

    def x_position(index):
        return (left + right) / 2 if len(points) == 1 else left + index * (right - left) / (len(points) - 1)

    for group, metric_specs, top, bottom, heading in [
        ('Startup', METRICS[:2], 127, 245, 110),
        ('Execution', METRICS[2:], 355, 485, 338),
    ]:
        values = [rows[key]['scriptw_ns'] / rows[key]['v8_ns']
                  for rows in all_rows for key, *_ in metric_specs]
        lower, upper, ticks = _scale(values)

        def y_position(value):
            return bottom - (math.log10(value) - lower) / (upper - lower) * (bottom - top)

        content.append(_text(28, heading, group, size=16, weight=600))
        _legend(content, metric_specs, 245 if group == 'Startup' else 270, heading)
        for exponent in ticks:
            y = bottom - (exponent - lower) / (upper - lower) * (bottom - top)
            content.append(_line(left, y, right, y))
            content.append(_text(left - 12, y + 4, _tick_label(exponent), size=11, fill=MUTED, anchor='end'))
        content.append(_line(left, y_position(1), right, y_position(1), stroke=INK, width=1.3, dash='4 4'))
        for index, point in enumerate(points):
            x = x_position(index)
            content.append(_line(x, top, x, bottom, stroke='#eaeef2'))
            measured = _timestamp(point['measured_at'])
            content.append(_text(x, bottom + 18, point['source_commit'][:9], size=10, anchor='middle'))
            content.append(_text(x, bottom + 32, measured.strftime('%Y-%m-%d'), size=10, fill=MUTED, anchor='middle'))
            content.append(_text(x, bottom + 46, measured.strftime('%H:%M UTC'), size=10, fill=MUTED, anchor='middle'))
        for key, label, _, color, dash in metric_specs:
            coords = [(x_position(index), y_position(rows[key]['scriptw_ns'] / rows[key]['v8_ns']))
                      for index, rows in enumerate(all_rows)]
            if len(coords) > 1:
                attributes = (f'points="{" ".join(f"{x:.3f},{y:.3f}" for x, y in coords)}" '
                              f'fill="none" stroke="{color}" stroke-width="2.6" stroke-linejoin="round"')
                if dash:
                    attributes += f' stroke-dasharray="{dash}"'
                content.append(f'<polyline {attributes}/>')
            for point, rows, (x, y) in zip(points, all_rows, coords, strict=True):
                ratio = rows[key]['scriptw_ns'] / rows[key]['v8_ns']
                content.append(f'<circle cx="{x:.3f}" cy="{y:.3f}" r="4.5" fill="{color}" '
                               f'data-source-commit="{escape(point["source_commit"], quote=True)}" '
                               f'data-report-sha256="{escape(point["id"], quote=True)}">'
                               f'<title>{escape(label)} · {escape(point["source_commit"][:9])}: '
                               f'{_ratio(ratio)}; Scriptw {_milliseconds(rows[key]["scriptw_ns"])} ms / '
                               f'V8 {_milliseconds(rows[key]["v8_ns"])} ms</title></circle>')
    content += [
        _text(28, 563, _context(latest), size=11, fill=MUTED),
        _text(28, 583, 'Only matched machine, runtime, workload and measurement policies are connected. Medians are not confidence intervals.',
              size=11, fill=MUTED),
        _text(28, 603, f'Latest: {latest["source_commit"][:9]} · cohort {latest["cohort_id"][:12]} · '
              'full history and raw milliseconds remain in the data.', size=11, fill=MUTED),
    ]
    description = ('Performance measured at matched compiler checkpoints. Startup and execution have separate log-scale panels. '
                   'V8 is a one-times elapsed-time reference; lower is better. X positions are equally spaced checkpoints, '
                   'not an elapsed calendar axis. ' + ('One checkpoint has no trend line. ' if len(points) == 1 else '')
                   + f'Shows {len(points)} of {total} checkpoints from one compatibility cohort.')
    return _svg('Scriptw versus V8 performance history', description, 623, content)
