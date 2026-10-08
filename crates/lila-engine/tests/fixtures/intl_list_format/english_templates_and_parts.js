function check(value, label) { if (!value) throw new Error(label); }
function same(actual, expected, label) { if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual); }
var rows = [
  ['conjunction', 'long', 'A and B', 'A, B, and C', 'A, B, C, and D', ['A', ', ', 'B', ', and ', 'C']],
  ['conjunction', 'short', 'A & B', 'A, B, & C', 'A, B, C, & D', ['A', ', ', 'B', ', & ', 'C']],
  ['conjunction', 'narrow', 'A, B', 'A, B, C', 'A, B, C, D', ['A', ', ', 'B', ', ', 'C']],
  ['disjunction', 'long', 'A or B', 'A, B, or C', 'A, B, C, or D', ['A', ', ', 'B', ', or ', 'C']],
  ['disjunction', 'short', 'A or B', 'A, B, or C', 'A, B, C, or D', ['A', ', ', 'B', ', or ', 'C']],
  ['disjunction', 'narrow', 'A or B', 'A, B, or C', 'A, B, C, or D', ['A', ', ', 'B', ', or ', 'C']],
  ['unit', 'long', 'A, B', 'A, B, C', 'A, B, C, D', ['A', ', ', 'B', ', ', 'C']],
  ['unit', 'short', 'A, B', 'A, B, C', 'A, B, C, D', ['A', ', ', 'B', ', ', 'C']],
  ['unit', 'narrow', 'A B', 'A B C', 'A B C D', ['A', ' ', 'B', ' ', 'C']]
];
for (var row of rows) {
  var lf = new Intl.ListFormat('en-US', {type: row[0], style: row[1]});
  same(lf.format([]), '', 'empty'); same(lf.format(['A']), 'A', 'singleton');
  same(lf.format(['A', 'B']), row[2], 'pair ' + row[0] + row[1]);
  same(lf.format(['A', 'B', 'C']), row[3], 'triple');
  same(lf.format(['A', 'B', 'C', 'D']), row[4], 'start/middle/end');
  var parts = lf.formatToParts(['A', 'B', 'C']);
  same(parts.length, 5, 'partition length');
  for (var i = 0; i < parts.length; i++) {
    same(parts[i].type, i % 2 === 0 ? 'element' : 'literal', 'partition kind');
    same(parts[i].value, row[5][i], 'partition value');
  }
  same(parts.map(function(p) { return p.value; }).join(''), row[3], 'one partition agrees with format');
  var resolved = lf.resolvedOptions(); same(resolved.type, row[0], 'resolved type'); same(resolved.style, row[1], 'resolved style');
}
print('ok english_templates_and_parts');
262;
