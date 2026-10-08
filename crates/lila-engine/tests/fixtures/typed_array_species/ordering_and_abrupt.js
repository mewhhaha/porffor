function require(value, label) { if (!value) throw label; }
function capture(callback, label) {
  try { callback(); } catch (error) { return error; }
  throw label + ' did not throw';
}
function invoke(method, source, name, callback, start, end) {
  if (name === 'map' || name === 'filter') return method.call(source, callback);
  return method.call(source, start, end);
}
var foreign = __lilaCreateRealm().global;
var errorPrototype = foreign.TypeError.prototype;
var methods = ['map', 'filter', 'slice', 'subarray'];
var marker = new foreign.TypeError('species marker');
foreign.TypeError = null;
globalThis.TypeError = null;
for (var name of methods) {
  var method = foreign.Uint8Array.prototype[name], trace = [], source = new Uint8Array([2, 3]);
  var handler = { construct: function(target, args, newTarget) {
    require(target === Target && newTarget === proxy && this === handler, 'construct identities');
    trace.push('construct:' + args.length);
    return name === 'subarray' ? new Uint8Array(args[0], args[1], args[2]) : new Uint8Array(args[0]);
  } };
  function Target() { throw 'trap bypassed'; }
  var proxy = new Proxy(Target, handler);
  var carrier = new Proxy({}, { get: function(target, key, receiver) {
    require(key === Symbol.species && receiver === carrier, 'species receiver');
    trace.push('species'); return proxy;
  } });
  Object.defineProperty(source, 'constructor', { get: function() {
    require(this === source, 'constructor receiver'); trace.push('constructor'); return carrier;
  } });
  var callback = function(value, index, actualSource) {
    require(actualSource === source, 'callback source'); trace.push('callback:' + index);
    return name === 'map' ? value : index === 1;
  };
  var start = { valueOf: function() { trace.push('start'); return 0; } };
  var end = { valueOf: function() { trace.push('end'); return 1; } };
  var result = invoke(method, source, name, callback, start, end);
  var expected = name === 'map' ? 'constructor,species,construct:1,callback:0,callback:1'
    : name === 'filter' ? 'callback:0,callback:1,constructor,species,construct:1'
    : name === 'slice' ? 'start,end,constructor,species,construct:1'
    : 'start,end,constructor,species,construct:3';
  require(trace.join(',') === expected, name + ' method order');
  require(result[0] === (name === 'filter' ? 3 : 2), 'selected result');

  // Every abrupt stage stops the rest, with filter's callbacks already complete.
  for (var stage of ['constructor', 'species', 'construct']) {
    var trace = [], source = new Uint8Array([1, 2]);
    function ThrowingSpecies() { trace.push('construct'); throw marker; }
    var carrier = {};
    Object.defineProperty(carrier, Symbol.species, { get: function() {
      trace.push('species'); if (stage === 'species') throw marker; return ThrowingSpecies;
    } });
    Object.defineProperty(source, 'constructor', { get: function() {
      trace.push('constructor'); if (stage === 'constructor') throw marker; return carrier;
    } });
    var assigned = 'unchanged';
    var error = capture(function() { const result = invoke(method, source, name, function(value, index) {
      trace.push('callback:' + index); return name === 'filter' ? true : value;
    }, 0, 2); assigned = result; }, stage);
    require(error === marker && Object.getPrototypeOf(error) === errorPrototype && assigned === 'unchanged', 'abrupt identity');
    var prefix = name === 'filter' ? 'callback:0,callback:1,' : '';
    var tail = stage === 'constructor' ? 'constructor' : stage === 'species' ? 'constructor,species' : 'constructor,species,construct';
    require(trace.join(',') === prefix + tail, name + ' abrupt stage order');
  }

  for (var invalid of [null, 1, true, 'constructor', Symbol('constructor'), 1n]) {
    var source = new Uint8Array([1]); source.constructor = invalid;
    var error = capture(function() { invoke(method, source, name, function(value) { return value; }, 0, 1); }, 'constructor value');
    require(Object.getPrototypeOf(error) === errorPrototype, 'constructor TypeError Realm');
  }
  for (var invalid of [false, 1, {}, () => 0]) {
    var source = new Uint8Array([1]); source.constructor = { [Symbol.species]: invalid };
    var error = capture(function() { invoke(method, source, name, function(value) { return value; }, 0, 1); }, 'species value');
    require(Object.getPrototypeOf(error) === errorPrototype, 'species TypeError Realm');
  }
  for (var failure of ['throw', 'nonobject', 'revoked']) {
    var trapCalls = 0;
    var handler = { construct: function() {
      trapCalls++; if (failure === 'throw') throw marker; return 1;
    } };
    var pair = Proxy.revocable(function SpeciesTarget() {}, handler);
    if (failure === 'revoked') pair.revoke();
    var source = new Uint8Array([1]); source.constructor = { [Symbol.species]: pair.proxy };
    var error = capture(function() { invoke(method, source, name, function(value) { return value; }, 0, 1); }, 'Proxy Construct');
    require(failure === 'throw' ? error === marker : Object.getPrototypeOf(error) === errorPrototype,
      'Proxy abrupt or execution Realm');
    require(trapCalls === (failure === 'revoked' ? 0 : 1), 'Proxy Construct failure order');
  }
  var proxyReads = 0;
  var wrongReceiver = new Proxy(new Uint8Array(1), { get: function() { proxyReads++; throw marker; } });
  var coercions = 0, offset = { valueOf: function() { coercions++; return 0; } };
  var error = capture(function() { invoke(method, wrongReceiver, name, function() { coercions++; }, offset, offset); }, 'receiver');
  require(Object.getPrototypeOf(error) === errorPrototype && proxyReads === 0 && coercions === 0, 'brand before hooks');
}
for (var name of ['map', 'filter']) {
  var method = foreign.Uint8Array.prototype[name], source = new Uint8Array(0), reads = 0;
  Object.defineProperty(source, 'constructor', { get: function() { reads++; throw marker; } });
  var error = capture(function() { method.call(source, null); }, 'callback');
  require(Object.getPrototypeOf(error) === errorPrototype && reads === 0, 'callback validation before species');
  var source = new Uint8Array([1, 2]), trace = [];
  Object.defineProperty(source, 'constructor', { get: function() { trace.push('constructor'); return undefined; } });
  var error = capture(function() { method.call(source, function() { trace.push('callback'); throw marker; }); }, 'callback throw');
  require(error === marker && trace.join(',') === (name === 'map' ? 'constructor,callback' : 'callback'), 'callback abrupt order');
}
for (var name of ['slice', 'subarray']) {
  var method = foreign.Uint8Array.prototype[name], source = new Uint8Array(2), trace = [];
  Object.defineProperty(source, 'constructor', { get: function() { trace.push('constructor'); throw 'late'; } });
  var error = capture(function() { method.call(source, { valueOf: function() { trace.push('start'); throw marker; } },
    { valueOf: function() { trace.push('end'); return 1; } }); }, 'start');
  require(error === marker && trace.join(',') === 'start', 'range abrupt before species');
}
print('typed-array-species-order:ok');
262;
