var methods = [Iterator.prototype.chunks, Iterator.prototype.windows];
for (var method of methods) {
  check(method.length === 1);
  var descriptor = Object.getOwnPropertyDescriptor(Iterator.prototype, method.name);
  check(descriptor.value === method && descriptor.writable && !descriptor.enumerable && descriptor.configurable);
  var threw = false;
  try { new method(1); } catch (error) { threw = error instanceof TypeError; }
  check(threw);
  for (var primitive of [undefined, null, true, 1, '', Symbol(), 1n]) {
    var threw = false;
    try { method.call(primitive, 1); } catch (error) { threw = error instanceof TypeError; }
    check(threw);
  }
  var conversions = 0;
  var log = [];
  var marker = {};
  var source = {get next() { log.push('next'); return function () { return {done: true}; }; }, get return() { log.push('return'); return function () { log.push('close'); throw marker; }; }};
  for (var invalid of [undefined, null, true, '2', 1n, Symbol(), new Number(2), {valueOf() { ++conversions; return 2; }}, NaN, Infinity, -Infinity, 1.5]) {
    log = [];
    var threw = false;
    try { method.call(source, invalid); } catch (error) { threw = error instanceof TypeError && error !== marker; }
    check(threw && log.join('|') === 'return|close');
  }
  for (var invalid of [0, -0, -1, 4294967296]) {
    log = [];
    var threw = false;
    try { method.call(source, invalid); } catch (error) { threw = error instanceof RangeError; }
    check(threw && log.join('|') === 'return|close');
  }
  check(conversions === 0);
  var gets = 0, closes = 0;
  var abrupt = {get next() { ++gets; throw marker; }, return() { ++closes; return {}; }};
  var threw = false;
  try { method.call(abrupt, 1); } catch (error) { threw = error === marker; }
  check(threw && gets === 1 && closes === 0);
  var deferred = method.call({next: 1, return() { ++closes; return {}; }}, 1);
  var threw = false;
  try { deferred.next(); } catch (error) { threw = error instanceof TypeError; }
  check(threw && deferred.next().done && deferred.return().done && closes === 0);
  check(method.call({next() { return {done: true}; }}, 4294967295).next().done);
}
var closes = 0, coercions = 0, nextGets = 0;
var source = {get next() { ++nextGets; return function () { return {done: true}; }; }, get return() { ++closes; throw 9; }};
for (var invalid of [null, true, 1, '', 'partial', new String('only-full'), {toString() { ++coercions; return 'only-full'; }}]) {
  var threw = false;
  try { Iterator.prototype.windows.call(source, 1, invalid); } catch (error) { threw = error instanceof TypeError; }
  check(threw);
}
check(closes === 7 && nextGets === 0 && coercions === 0);
check(Iterator.prototype.windows.call(source, 1, undefined).next().done && nextGets === 1);
