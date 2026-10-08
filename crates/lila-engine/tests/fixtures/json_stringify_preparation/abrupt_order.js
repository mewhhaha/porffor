function check(label, body) {
  var thrown = {};
  var trace = [];
  var same = false;
  try { body(thrown, trace); }
  catch (error) { same = error === thrown; }
  print(label + ':' + same + ':' + trace.join(','));
}

check('callable-first', function (thrown, trace) {
  var revoked = Proxy.revocable(function () {}, {});
  revoked.revoke();
  var value = {
    toJSON: function () { trace.push('toJSON'); throw thrown; }
  };
  JSON.stringify(value, revoked.proxy);
});

var arrayTrace = [];
var revokedArray = Proxy.revocable([], {});
revokedArray.revoke();
var arrayError = false;
try {
  JSON.stringify({ toJSON: function () { arrayTrace.push('toJSON'); } }, revokedArray.proxy);
} catch (error) { arrayError = error instanceof TypeError; }
print('noncallable-array:' + arrayError + ':' + arrayTrace.join(','));

check('index', function (thrown, trace) {
  var list = new Array(2);
  var prototype = Object.create(Array.prototype);
  Object.defineProperty(prototype, '0', {
    get: function () { trace.push('index:0'); throw thrown; }
  });
  Object.setPrototypeOf(list, prototype);
  Object.defineProperty(list, '1', {
    get: function () { trace.push('index:1'); return 'a'; }
  });
  JSON.stringify({ toJSON: function () { trace.push('toJSON'); } }, list);
});

check('wrapper', function (thrown, trace) {
  var key = new String('a');
  key[Symbol.toPrimitive] = function (hint) {
    trace.push('convert:' + hint);
    throw thrown;
  };
  var list = new Array(2);
  Object.defineProperty(list, '0', {
    get: function () { trace.push('index:0'); return key; }
  });
  Object.defineProperty(list, '1', {
    get: function () { trace.push('index:1'); return 'b'; }
  });
  JSON.stringify({ toJSON: function () { trace.push('toJSON'); } }, list);
});

check('toJSON-get', function (thrown, trace) {
  var value = [];
  Object.defineProperty(value, 'toJSON', {
    get: function () { trace.push('get'); throw thrown; }
  });
  JSON.stringify(value, function (key, value) { trace.push('replacer'); return value; });
});

check('toJSON-call', function (thrown, trace) {
  function value() {}
  Object.defineProperty(value, 'toJSON', {
    get: function () {
      trace.push('get');
      return function (key) {
        trace.push('call:' + (key === '') + ':' + (this === value));
        throw thrown;
      };
    }
  });
  JSON.stringify(value, function (key, value) { trace.push('replacer'); return value; });
});

check('unbox', function (thrown, trace) {
  var boxed = new Number(0);
  boxed[Symbol.toPrimitive] = function (hint) { trace.push('coerce:' + hint); throw thrown; };
  var value = { toJSON: function () { trace.push('toJSON'); return boxed; } };
  JSON.stringify(value, function (key, value) { trace.push('replacer'); return value; });
});

check('replacer', function (thrown, trace) {
  var value = { toJSON: function () { trace.push('toJSON'); return 1; } };
  JSON.stringify(value, function () { trace.push('replacer'); throw thrown; });
});
