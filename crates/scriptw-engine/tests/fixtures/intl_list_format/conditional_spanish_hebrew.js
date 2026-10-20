function same(actual, expected, label) { if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual); }
function pair(lf, left, right, joiner) {
  var expected = left + joiner + right;
  same(lf.format([left, right]), expected, 'conditional pair');
  var parts = lf.formatToParts([left, right]);
  same(parts.length, 3, 'conditional partition'); same(parts[0].value, left, 'original left'); same(parts[1].type, 'literal', 'conditional literal'); same(parts[1].value, joiner, 'selected joiner'); same(parts[2].value, right, 'original right');
}
var and = new Intl.ListFormat('es', {type: 'conjunction', style: 'long'});
for (var word of ['iglesia', 'Iglesia', 'hilo', 'HI', 'hi']) pair(and, 'agua', word, ' e ');
for (var word of ['hielo', 'hiato', 'HIA', 'otro', '']) pair(and, 'agua', word, ' y ');
same(and.format(['agua', 'pan', 'iglesia']), 'agua, pan e iglesia', 'conditional end after start');
var or = new Intl.ListFormat('es', {type: 'disjunction', style: 'long'});
for (var word of ['oso', 'HOJA', '8', '80', '11', '11.000', '11 000', '11\u202f000', '11,2']) pair(or, 'siete', word, ' u ');
for (var word of ['110', '1100', '211000', 'algo', '']) pair(or, 'siete', word, ' o ');
same(or.format(['uno', 'dos', '8']), 'uno, dos u 8', 'conditional disjunction end');
var he = new Intl.ListFormat('he', {type: 'conjunction', style: 'long'});
pair(he, '\u05d0', '\u05d1', ' \u05d5');
pair(he, '\u05d0', 'A', ' \u05d5\u2011');
pair(he, '\u05d0', '', ' \u05d5');
pair(he, '\u05d0', '\ud800', ' \u05d5\u2011');
pair(he, '\u05d0', '\ud83d\ude00', ' \u05d5\u2011');
same(he.format(['\u05d0', '\u05d1', 'A']), '\u05d0, \u05d1 \u05d5\u2011A', 'Hebrew contextual end');
print('ok conditional_spanish_hebrew');
262;
