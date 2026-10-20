function check(value, label) { if (!value) throw new Error(label); }
function same(actual, expected, label) { if (!Object.is(actual, expected)) throw new Error(label); }
var lf = new Intl.ListFormat('en-US');
for (var method of ['format', 'formatToParts']) {
  for (var fault of ['iterator.get', 'iterator.call', 'next.get', 'next.call', 'done.get', 'value.get']) {
    var marker = {}, closes = 0, nexts = 0, values = 0;
    var iterator = {get next() {
      if (fault === 'next.get') throw marker;
      return function() {
        nexts++; if (fault === 'next.call') throw marker;
        return {get done() { if (fault === 'done.get') throw marker; return false; }, get value() { values++; throw marker; }};
      };
    }, get return() { closes++; throw new Error('operation abrupt must not close'); }};
    var iterable = {get [Symbol.iterator]() { if (fault === 'iterator.get') throw marker; return function() { if (fault === 'iterator.call') throw marker; return iterator; }; }};
    var caught = undefined;
    try { lf[method](iterable); } catch (e) { caught = e; }
    same(caught, marker, fault + ' exact abrupt'); same(closes, 0, fault + ' no return Get');
    same(nexts, fault === 'next.call' || fault === 'done.get' || fault === 'value.get' ? 1 : 0, fault + ' next count');
    same(values, fault === 'value.get' ? 1 : 0, fault + ' value count');
  }
  for (var invalid of [null, 1, {}]) {
    var error = undefined;
    try { lf[method](invalid); } catch (e) { error = e; }
    check(error && error.constructor === TypeError, 'noniterable input rejected');
  }
}
print('ok iterator_abrupts_do_not_close');
262;
