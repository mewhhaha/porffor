function run(value, carrier, label) {
  var trace = [];
  var boxed = new Number(0);
  boxed[Symbol.toPrimitive] = function (hint) {
    trace.push('unbox:' + hint + ':' + (this === boxed));
    return 7;
  };
  Object.defineProperty(carrier, 'toJSON', {
    get: function () {
      trace.push('get:' + (this === value));
      return function (key) {
        trace.push('call:' + (key === '') + ':' + (this === value));
        return boxed;
      };
    }
  });
  var result = JSON.stringify(value, function (key, replacement) {
    trace.push('replace:' + (key === '') + ':' + (replacement === boxed));
    return replacement;
  });
  print(label + ':' + result + ':' + trace.join(','));
}

var ownArray = [];
run(ownArray, ownArray, 'array-own');
var inheritedArray = [];
var prototype = Object.create(Array.prototype);
Object.setPrototypeOf(inheritedArray, prototype);
run(inheritedArray, prototype, 'array-inherited');
function callableValue() {}
run(callableValue, callableValue, 'function');
function argumentsValue() { return arguments; }
var args = argumentsValue(1);
run(args, args, 'arguments');

var nestedTrace = [];
function child() {}
Object.defineProperty(child, 'toJSON', {
  get: function () {
    nestedTrace.push('get:' + (this === child));
    return function (key) {
      nestedTrace.push('call:' + key + ':' + (this === child));
      return 9;
    };
  }
});
var nested = { child: child, items: [child] };
var text = JSON.stringify(nested, function (key, value) {
  nestedTrace.push('replace:' + key);
  return value;
});
print(text);
print(nestedTrace.join(','));
