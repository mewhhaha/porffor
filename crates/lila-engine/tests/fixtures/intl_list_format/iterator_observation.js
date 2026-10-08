function check(value, label) { if (!value) throw new Error(label); }
function same(actual, expected, label) { if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual); }
var lf = new Intl.ListFormat('en-US');
for (var method of ['format', 'formatToParts']) {
  var log = [], count = 0, nextReads = 0, closeReads = 0;
  var iterator = {get next() {
    nextReads++; log.push('get next');
    return function() {
      same(this, iterator, 'next receiver'); same(arguments.length, 0, 'next has no args');
      var n = count++; log.push('next ' + n);
      if (n === 0) Object.defineProperty(iterator, 'next', {value() { throw new Error('next was not cached'); }, configurable: true});
      return {get done() { log.push('done ' + n); return n === 2 ? {} : 0; }, get value() { log.push('value ' + n); if (n === 2) throw new Error('done result value read'); return n === 0 ? 'A' : 'B'; }};
    };
  }, get return() { closeReads++; throw new Error('normal exhaustion must not close'); }};
  var iterable = {get [Symbol.iterator]() {
    log.push('get iterator');
    return function() { same(this, iterable, 'iterator method receiver'); same(arguments.length, 0, 'iterator method has no args'); log.push('iterator call'); return iterator; };
  }};
  var value = lf[method](iterable);
  same(method === 'format' ? value : value.map(function(p) { return p.value; }).join(''), 'A and B', 'observed strings format');
  same(log.join('|'), 'get iterator|iterator call|get next|next 0|done 0|value 0|next 1|done 1|value 1|next 2|done 2', 'iterator protocol order');
  same(nextReads, 1, 'next cached once'); same(closeReads, 0, 'no normal close'); same(count, 3, 'one final next');
}
print('ok iterator_observation');
262;
