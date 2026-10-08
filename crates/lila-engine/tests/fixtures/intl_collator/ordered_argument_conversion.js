function check(value, label) { if (!value) throw new Error(label); }
function same(actual, expected, label) { if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual); }
function throws(C, body, label) { var caught; try { body(); } catch (e) { caught = e; } check(caught && caught.constructor === C && caught instanceof C, label); return caught; }
var cmp = new Intl.Collator('en-US').compare; var log = [];
var x = {toString() { log.push('x'); return 'a'; }}; var y = {toString() { log.push('y'); return 'b'; }};
check(cmp(x, y) < 0, 'native ordering'); same(log.join('|'), 'x|y', 'ordered conversions');
var marker = {}; log = []; x = {toString() { log.push('x'); throw marker; }}; y = {toString() { log.push('y'); return 'b'; }};
var caught; try { cmp(x, y); } catch (e) { caught = e; } same(caught, marker, 'x abrupt identity'); same(log.join('|'), 'x', 'y not coerced');
same(cmp(), 0, 'two missing undefined'); same(cmp(undefined, 'undefined'), 0, 'one missing converted');
log = []; throws(TypeError, function() { cmp(Symbol('x'), y); }, 'Symbol ToString throws'); same(log.length, 0, 'y after Symbol not coerced');
print('ok ordered_argument_conversion');
262;
