function check(value, label) { if (!value) throw new Error(label); }
function same(actual, expected, label) { if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual); }
function throws(C, body, label) { var caught; try { body(); } catch (e) { caught = e; } check(caught && caught.constructor === C && caught instanceof C, label); return caught; }
var keys = ['usage', 'localeMatcher', 'collation', 'numeric', 'caseFirst', 'sensitivity', 'ignorePunctuation'];
var values = {usage: 'sort', localeMatcher: 'lookup', collation: 'default', numeric: false, caseFirst: 'false', sensitivity: 'variant', ignorePunctuation: false};
for (var i = 0; i < keys.length; i++) { var marker = {}; var log = []; var stop = keys[i]; var opts = new Proxy({}, {get(t, k) { log.push(k); if (k === stop) throw marker; return values[k]; }}); var caught; try { new Intl.Collator('en-US', opts); } catch (e) { caught = e; } same(caught, marker, 'getter throw identity'); same(log.join('|'), keys.slice(0, i + 1).join('|'), 'later getters stopped'); }
var seen = []; var bad = {get usage() { seen.push('usage'); return 'sort'; }, get localeMatcher() { seen.push('localeMatcher'); return 'lookup'; }, get collation() { seen.push('collation'); return 'a'; }, get numeric() { throw new Error('numeric before collation validation'); }};
throws(RangeError, function() { new Intl.Collator('en-US', bad); }, 'collation syntax first'); same(seen.join('|'), 'usage|localeMatcher|collation', 'syntax stop order');
var touched = 0; function T() {} var nt = new Proxy(T, {get(t, k) { if (k === 'prototype') throw marker; return Reflect.get(t, k); }}); var loc = {toString() { touched++; return 'en-US'; }};
try { Reflect.construct(Intl.Collator, [[loc], {}], nt); } catch (e) { same(e, marker, 'prototype abrupt identity'); } same(touched, 0, 'prototype first');
print('ok constructor_short_circuits');
262;
