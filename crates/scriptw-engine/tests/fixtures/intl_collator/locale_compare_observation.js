function check(value, label) { if (!value) throw new Error(label); }
function same(actual, expected, label) { if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual); }
function throws(C, body, label) { var caught; try { body(); } catch (e) { caught = e; } check(caught && caught.constructor === C && caught instanceof C, label); return caught; }
var log = []; var receiver = {toString() { log.push('this'); return '2'; }}; var that = {toString() { log.push('that'); return '10'; }};
var locale = {toString() { log.push('locale'); return 'en-US'; }}; var opts = {get usage() { log.push('usage'); return 'sort'; }, get numeric() { log.push('numeric'); return true; }, get sensitivity() { log.push('sensitivity'); return 'variant'; }, get ignorePunctuation() { log.push('ignore'); return false; }};
check(String.prototype.localeCompare.call(receiver, that, [locale], opts) < 0, 'actual numeric consumer'); same(log.join('|'), 'this|that|locale|usage|numeric|sensitivity|ignore', 'immutable Construct after two strings');
var marker = {}; log = []; var thrown = {toString() { log.push('that'); throw marker; }}; var caught; try { String.prototype.localeCompare.call(receiver, thrown, [locale], opts); } catch (e) { caught = e; } same(caught, marker, 'that abrupt identity'); same(log.join('|'), 'this|that', 'locales after both strings');
log = []; throws(TypeError, function() { String.prototype.localeCompare.call(null, that, [locale], opts); }, 'coercible receiver first'); same(log.length, 0, 'null before that');
print('ok locale_compare_observation');
262;
