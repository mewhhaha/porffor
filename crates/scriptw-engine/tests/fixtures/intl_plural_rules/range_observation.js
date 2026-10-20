// Source/data-derived expectations; authored control, not an executed result.
function check(value, label) { if (!value) throw new Error(label); }
function same(actual, expected, label) { check(Object.is(actual, expected), label); }
function throws(ctor, action, label) {
  var caught;
  try { action(); } catch (error) { caught = error; }
  check(caught instanceof ctor && caught.constructor === ctor, label);
}
function abrupt(marker, action, label) {
  var caught;
  try { action(); } catch (error) { caught = error; }
  same(caught, marker, label);
}

var rules = new Intl.PluralRules('en'), log = [], marker = {}, endMarker = {};
var start = {valueOf() {log.push('start'); return 1;}}, end = {valueOf() {log.push('end'); return 2;}};
throws(TypeError, function () { rules.selectRange(undefined,end); }, 'undefined start');
throws(TypeError, function () { rules.selectRange(start,undefined); }, 'undefined end'); same(log.length, 0, 'undefined before both conversions');
throws(TypeError, function () { Intl.PluralRules.prototype.selectRange.call({},start,end); }, 'range brand'); same(log.length, 0, 'range brand before input');
abrupt(marker, function () { rules.selectRange({valueOf() {log.push('start');throw marker;}},end); }, 'first abrupt'); same(log.join(','), 'start', 'first abrupt stops end');
log = [];
abrupt(endMarker, function () { rules.selectRange({valueOf() {log.push('start');return NaN;}},{valueOf() {log.push('end');throw endMarker;}}); }, 'NaN start still converts end'); same(log.join(','), 'start,end', 'both conversions before NaN decision');
log = [];
throws(RangeError, function () { rules.selectRange({valueOf() {log.push('start');return NaN;}},end); }, 'NaN range'); same(log.join(','), 'start,end', 'NaN RangeError after end');
log = [];
same(rules.selectRange(start,end), 'other', 'finite range'); same(log.join(','), 'start,end', 'one conversion each');
for (var pair of [[2,1],[-2,-1],[Infinity,-Infinity],[Infinity,Infinity]]) same(rules.selectRange(pair[0],pair[1]), 'other', 'no descending negative or infinity rejection');

print("ok range_observation");
262;
