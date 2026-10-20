import copy
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location('test_graph', ROOT / 'scripts/generate-test-graph.py')
GRAPH = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GRAPH)


class RecordedTestChart(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.publication = json.loads(GRAPH.read_committed_or_local(ROOT, GRAPH.SNAPSHOT))

    def setUp(self):
        self.record = copy.deepcopy(self.publication)

    def test_gray_is_not_implemented_and_red_includes_bugs_and_crashes(self):
        self.assertEqual([row[2] for row in GRAPH.categories(self.record)], [96678, 4240, 1125])
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

    def test_svg_has_accessible_linear_zero_based_bars_in_the_requested_colors(self):
        svg = ET.fromstring(GRAPH.render_svg(self.record))
        self.assertEqual(svg.attrib['role'], 'img')
        ns = {'svg': 'http://www.w3.org/2000/svg'}
        self.assertIn('Historical snapshot', svg.find('svg:desc', ns).text)
        bars = {node.attrib['id']: node for node in svg.findall('svg:g/svg:rect', ns)}
        scale = None
        for key, _, count, color in GRAPH.categories(self.record):
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
