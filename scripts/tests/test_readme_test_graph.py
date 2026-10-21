import copy
import importlib.util
import json
from pathlib import Path
import sys
import tempfile
import unittest
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))
SPEC = importlib.util.spec_from_file_location('test_graph', ROOT / 'scripts/generate-test-graph.py')
GRAPH = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GRAPH)


class RecordedTestChart(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        # A stable historical fixture; unit tests must survive a new publication.
        # The actual committed output is separately verified by --check in CI.
        cls.publication = {'real_suite': {
            'refresh_date': '2026-10-02', 'total': 102043, 'passed': 96678,
            'pinned_revisions': {'test262': 'aa55200d1310384c5cf69ea95b2a2ecba457007b'},
            'counts_per_outcome': [
                {'label': 'Success', 'count': 96678},
                {'label': 'Bug', 'count': 4201},
                {'label': 'Crash', 'count': 39},
                {'label': 'NotImplemented', 'count': 1125},
            ],
        }}

    def setUp(self):
        self.record = copy.deepcopy(self.publication)

    def test_gray_is_not_implemented_and_red_includes_bugs_and_crashes(self):
        self.assertEqual([row[2] for row in GRAPH.categories(self.record)], [96678, 4240, 1125, None])
        text = GRAPH.render(self.record)
        self.assertIn('1,125 not implemented', text)
        self.assertIn('4,240 bugs/crashes', text)
        self.assertIn('gray means unimplemented, not unrun', text)
        self.assertIn('2026-10-02', text)
        self.assertIn('does not establish\ncurrent compiler conformance', text)
        self.record['real_suite']['counts_per_outcome'].append({'label': 'Unrun', 'count': 0})
        with self.assertRaisesRegex(ValueError, 'unknown published outcomes'):
            GRAPH.render(self.record)

    def test_inconsistent_duplicate_negative_and_noninteger_outcomes_are_rejected(self):
        for mutate, error in [
            (lambda r: r['real_suite'].update(total=1), 'published totals disagree'),
            (lambda r: r['real_suite'].update(passed=True), 'published totals disagree'),
            (lambda r: r['real_suite']['counts_per_outcome'].append(
                r['real_suite']['counts_per_outcome'][0].copy()), 'duplicate'),
            (lambda r: r['real_suite']['counts_per_outcome'][0].update(count=-1), 'nonnegative'),
            (lambda r: r['real_suite']['counts_per_outcome'][0].update(count=True), 'nonnegative'),
        ]:
            record = copy.deepcopy(self.publication)
            mutate(record)
            with self.subTest(error=error), self.assertRaisesRegex(ValueError, error):
                GRAPH.render_svg(record)

    def test_source_bound_publication_uses_execution_units_and_measured_revision(self):
        self.record['real_suite']['compiler_identity'] = {
            'source_revision': {'kind': 'git-commit', 'commit': 'a' * 40},
        }
        self.record['real_suite']['refresh_date'] = '2026-10-20'
        text = GRAPH.render(self.record)
        self.assertIn('mode-specific executions', text)
        self.assertIn('Measured compiler: `aaaaaaaaa`', text)
        self.assertIn('2026-10-20', text)
        self.assertNotIn('historical snapshot', text)
        self.assertNotIn('path-based entries', text)
        svg = GRAPH.render_svg(self.record)
        self.assertIn('Complete pinned-suite baseline', svg)
        self.assertIn('number of test executions', svg)
        self.assertNotIn('Historical snapshot', svg)

    def test_svg_has_accessible_linear_zero_based_bars_in_the_requested_colors(self):
        svg = ET.fromstring(GRAPH.render_svg(self.record))
        self.assertEqual(svg.attrib['role'], 'img')
        ns = {'svg': 'http://www.w3.org/2000/svg'}
        self.assertIn('Historical snapshot', svg.find('svg:desc', ns).text)
        bars = {node.attrib['id']: node for node in svg.findall('svg:g/svg:rect', ns)}
        scale = None
        for key, _, count, color in GRAPH.categories(self.record):
            if count is None:
                continue
            bar = bars[key].attrib
            self.assertEqual(int(bar['data-count']), count)
            self.assertEqual(bar['fill'], color)
            height = float(bar['height'])
            self.assertGreater(height, 0)
            self.assertAlmostEqual(float(bar['y']) + height, 326, places=5)
            if scale is None:
                scale = height / count
            self.assertAlmostEqual(height / count, scale, places=8)
        self.assertNotIn('<script', GRAPH.render_svg(self.record))

    def test_unversioned_build_is_identified_by_fingerprint_and_unknown_kind_is_rejected(self):
        self.record['real_suite']['compiler_identity'] = {
            'source_revision': {'kind': 'unversioned-archive'},
            'source_fingerprint': 'b' * 64,
        }
        self.assertIn('Measured compiler: `archive bbbbbbbbb`', GRAPH.render(self.record))
        self.record['real_suite']['compiler_identity']['source_revision']['kind'] = 'invented'
        with self.assertRaisesRegex(ValueError, 'unknown compiler source revision'):
            GRAPH.render(self.record)

    def test_axis_does_not_double_at_one_hundred_thousand_passes(self):
        real = self.record['real_suite']
        real['counts_per_outcome'][0]['count'] = real['passed'] = 101000
        real['total'] = sum(row['count'] for row in real['counts_per_outcome'])
        svg = ET.fromstring(GRAPH.render_svg(self.record))
        ns = {'svg': 'http://www.w3.org/2000/svg'}
        bar = svg.find('svg:g/svg:rect[@id="passing"]', ns)
        self.assertGreater(float(bar.attrib['height']), 0.75 * 248)
        self.assertLessEqual(float(bar.attrib['height']), 248)

    def test_out_of_scope_is_unknown_not_zero_or_a_guess_from_unsupported_counts(self):
        rows = GRAPH.categories(self.record)
        self.assertIsNone(rows[-1][2])
        self.assertEqual(sum(count for _, _, count, _ in rows if count is not None), 102043)
        text = GRAPH.render(self.record)
        self.assertIn('out of scope not classified', text)
        self.assertIn('count is **unknown**, not zero', text)
        self.assertIn('published totals\nabove remain unchanged', text)
        svg = ET.fromstring(GRAPH.render_svg(self.record))
        ns = {'svg': 'http://www.w3.org/2000/svg'}
        unknown = svg.find('svg:g/svg:g[@id="out-of-scope"]', ns)
        self.assertEqual(unknown.attrib['data-status'], 'unclassified')
        self.assertEqual(unknown.attrib['fill'], '#0969da')
        self.assertNotIn('data-count', unknown.attrib)
        self.assertEqual(unknown.findall('svg:rect', ns), [])
        self.assertIn('Out of scope: not classified, count unknown', svg.find('svg:desc', ns).text)
        # FailureKind::Unsupported is not a scope decision; no subtraction.
        self.record['real_suite']['counts_per_kind'] = [{'label': 'Unsupported', 'count': 1125}]
        self.assertEqual(GRAPH.categories(self.record), rows)

    def test_check_detects_stale_svg_and_readme_and_keeps_current_files_untouched(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            snapshot = root / GRAPH.SNAPSHOT
            snapshot.parent.mkdir(parents=True)
            snapshot.write_text(json.dumps(self.record))
            readme = root / 'README.md'
            readme.write_text('pitch\n' + GRAPH.START + '\n' + GRAPH.END + '\ncaveats\n')
            self.assertEqual(len(GRAPH.generate(root)), 2)
            chart = root / GRAPH.CHART
            before = {p: (p.read_bytes(), p.stat().st_mtime_ns) for p in [readme, chart]}
            self.assertEqual(GRAPH.generate(root, check=True), [])
            self.assertEqual(before, {p: (p.read_bytes(), p.stat().st_mtime_ns) for p in before})
            self.assertTrue(readme.read_text().startswith('pitch\n'))
            self.assertTrue(readme.read_text().endswith('\ncaveats\n'))
            chart.write_text(chart.read_text() + ' ')
            with self.assertRaisesRegex(ValueError, 'test262-status.svg'):
                GRAPH.generate(root, check=True)
            self.assertEqual(readme.read_bytes(), before[readme][0])
            self.assertEqual(GRAPH.generate(root), [chart])
            readme.write_text(readme.read_text().replace('1,125 not implemented', '0 not implemented'))
            with self.assertRaisesRegex(ValueError, 'README.md'):
                GRAPH.generate(root, check=True)

    def test_missing_duplicate_and_reversed_markers_cannot_rewrite_the_page(self):
        for text in ['', GRAPH.START + GRAPH.START + GRAPH.END, GRAPH.END + GRAPH.START]:
            with self.subTest(text=text), self.assertRaisesRegex(ValueError, 'one ordered'):
                GRAPH.replace_block(text, 'replacement')
