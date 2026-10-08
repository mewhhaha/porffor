function require(value, label) { if (!value) throw label; }
function invoke(method, source, name) {
  if (name === 'map') return method.call(source, function(value) { return value; });
  if (name === 'filter') return method.call(source, function() { return true; });
  return method.call(source, 0, 2);
}
var foreign = __lilaCreateRealm().global;
var names = ['Float16Array', 'Float32Array', 'Float64Array', 'Int8Array',
  'Int16Array', 'Int32Array', 'Uint8Array', 'Uint8ClampedArray', 'Uint16Array',
  'Uint32Array', 'BigInt64Array', 'BigUint64Array'];
var local = [], remote = [], methodNames = ['map', 'filter', 'slice', 'subarray'];
var LocalBuffer = ArrayBuffer, localBufferPrototype = ArrayBuffer.prototype;
var foreignBufferPrototype = foreign.ArrayBuffer.prototype;
for (var name of names) { local.push(globalThis[name]); remote.push(foreign[name]); }
function Replacement() { throw 'public constructor consulted'; }
for (var index = 0; index < names.length; index++) {
  globalThis[names[index]] = Replacement;
  foreign[names[index]] = Replacement;
  local[index].prototype.constructor = Replacement;
  remote[index].prototype.constructor = Replacement;
}
globalThis.ArrayBuffer = Replacement;
foreign.ArrayBuffer = Replacement;
var defaults = [undefined, { [Symbol.species]: undefined }, { [Symbol.species]: null }];
for (var index = 0; index < names.length; index++) {
  var isBigInt = index >= 10, values = isBigInt ? [1n, 2n] : [1, 2];
  for (var methodName of methodNames) {
    for (var choice of defaults) {
      var source = new local[index](values);
      source.constructor = choice;
      var result = invoke(remote[index].prototype[methodName], source, methodName);
      require(Object.getPrototypeOf(result) === remote[index].prototype, names[index] + ' foreign default');
      require(result.length === 2 && result[0] === values[0] && result[1] === values[1], 'default values');
      if (methodName === 'subarray') require(result.buffer === source.buffer, 'supplied source buffer');
      else require(Object.getPrototypeOf(result.buffer) === foreignBufferPrototype && result.buffer !== source.buffer,
        'foreign constructor backing');
      var source = new remote[index](values);
      source.constructor = choice;
      var result = invoke(local[index].prototype[methodName], source, methodName);
      require(Object.getPrototypeOf(result) === local[index].prototype, names[index] + ' entry default');
      require(result[0] === values[0] && result[1] === values[1], 'entry default values');
      if (methodName !== 'subarray') require(Object.getPrototypeOf(result.buffer) === localBufferPrototype, 'entry backing');
    }
  }
}

// Array, Function and Proxy constructor carriers retain their actual Get route.
var Uint = local[6];
for (var methodName of methodNames) {
  for (var form = 0; form < 3; form++) {
    var source = new Uint([7, 9]), observedArguments, constructCalls = 0, carrierGets = 0;
    function Target() { throw 'construct trap bypassed'; }
    var handler = { construct: function(target, args, newTarget) {
      require(this === handler && target === Target && newTarget === proxy, 'Proxy Construct identities');
      constructCalls++;
      observedArguments = args;
      return methodName === 'subarray' ? new Uint(args[0], args[1], args[2]) : new Uint(args[0]);
    } };
    var proxy = new Proxy(Target, handler);
    var carrier;
    if (form === 0) { carrier = []; carrier[Symbol.species] = proxy; }
    else if (form === 1) { carrier = function Carrier() {}; carrier[Symbol.species] = proxy; }
    else { carrier = new Proxy({}, { get: function(target, key, receiver) {
      require(key === Symbol.species && receiver === carrier, 'Proxy carrier receiver');
      carrierGets++; return proxy;
    } }); }
    source.constructor = carrier;
    var result = invoke(Uint.prototype[methodName], source, methodName);
    require(constructCalls === 1 && carrierGets === (form === 2 ? 1 : 0), 'single species construction');
    require(result[0] === 7 && result[1] === 9, 'custom species values');
    if (methodName === 'subarray') require(observedArguments.length === 3 && observedArguments[0] === source.buffer &&
      observedArguments[1] === 0 && observedArguments[2] === 2, 'custom buffer argument vector');
    else require(observedArguments.length === 1 && observedArguments[0] === 2, 'custom numeric argument vector');
  }
}
require(new LocalBuffer(1).byteLength === 1, 'saved buffer constructor still works');
print('typed-array-species-constructors:ok');
262;
