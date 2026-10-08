var cursor = 0;
var iterator = {
  get next() {
    print('next-get');
    return function () {
      var index = cursor++; print('next:' + index);
      return { get done() { print('done:' + index); return index === 2; },
        get value() { print('value:' + index); return (index + 1) * 10; } };
    };
  },
  return: function () { print('wrong-close'); return {}; }
};
var source = {};
Object.defineProperty(source, Symbol.iterator, { configurable: true, get: function () {
  print('iterator-get'); return function () { print('iterator-call'); return iterator; };
} });
var tail = {}; tail[Symbol.iterator] = function () {
  print('tail'); var index = 0;
  return { next: function () { return { done: index++ !== 0, value: 30 }; } };
};
function lead() { print('lead'); return 5; }
function operand() {
  print('operand');
  return Promise.resolve(99).then(function (value) {
    print('replace');
    Object.defineProperty(source, Symbol.iterator, { value: function () { print('wrong-reiterate'); throw 0; } });
    Array.prototype[Symbol.iterator] = function () { print('wrong-private-iterator'); throw 0; };
    return value;
  });
}
function consume(a, b, c, d, e) { print('values:' + a + ':' + b + ':' + c + ':' + d + ':' + e); }
async function run() { consume(lead(), ...source, await operand(), ...tail); print('done'); }
run().catch(error => print('error:' + error)); print('called');
