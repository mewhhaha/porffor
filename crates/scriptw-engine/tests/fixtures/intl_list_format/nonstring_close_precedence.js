function check(value, label) { if (!value) throw new Error(label); }
function same(actual, expected, label) { if (!Object.is(actual, expected)) throw new Error(label); }
var lf = new Intl.ListFormat('en-US');
for (var method of ['format', 'formatToParts']) {
  for (var closing of ['getter.throw', 'absent', 'not.callable', 'call.throw', 'primitive', 'object']) {
    var other = {}, reads = 0, calls = 0, nexts = 0, conversions = 0;
    var bad = {toString() { conversions++; throw new Error('nonstring must not be coerced'); }, [Symbol.toPrimitive]() { conversions++; throw new Error('nonstring must not be coerced'); }};
    var iterator = {next() { nexts++; return {done: false, value: nexts === 1 ? 'A' : bad}; }, get return() {
      reads++;
      if (closing === 'getter.throw') throw other;
      if (closing === 'absent') return undefined;
      if (closing === 'not.callable') return 42;
      return function() { calls++; same(this, iterator, 'return receiver'); same(arguments.length, 0, 'return no args'); if (closing === 'call.throw') throw other; return closing === 'primitive' ? 1 : {}; };
    }};
    var caught = undefined;
    try { lf[method]({[Symbol.iterator]() { return iterator; }}); } catch (e) { caught = e; }
    check(caught && caught.constructor === TypeError, 'initial TypeError wins ' + closing);
    check(caught !== other, 'close failure cannot replace initial throw');
    same(reads, 1, 'return getter once');
    same(calls, closing === 'call.throw' || closing === 'primitive' || closing === 'object' ? 1 : 0, 'return call count');
    same(nexts, 2, 'stop immediately at nonstring'); same(conversions, 0, 'no coercion hooks');
  }
  for (var bad of [1, 1n, Symbol('item'), null, undefined, false, new String('A')]) {
    var closes = 0;
    var iterator = {next() { return {done: false, value: bad}; }, return() { closes++; return {}; }};
    var caught = undefined;
    try { lf[method]({[Symbol.iterator]() { return iterator; }}); } catch (e) { caught = e; }
    check(caught && caught.constructor === TypeError, 'only primitive strings admitted'); same(closes, 1, 'each nonstring closes');
  }
}
print('ok nonstring_close_precedence');
262;
