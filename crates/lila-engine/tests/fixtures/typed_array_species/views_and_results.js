function require(value, label) { if (!value) throw label; }
function capture(callback, label) {
  try { callback(); } catch (error) { return error; }
  throw label + ' did not throw';
}
function invoke(method, source, name) {
  if (name === 'map') return method.call(source, function(value) { return value; });
  if (name === 'filter') return method.call(source, function() { return true; });
  return method.call(source, 0, source.length);
}
var foreign = __lilaCreateRealm().global, errorPrototype = foreign.TypeError.prototype;
var names = ['map', 'filter', 'slice', 'subarray'];
for (var name of names) {
  var method = foreign.Uint8Array.prototype[name];
  for (var kind of ['object', 'proxy', 'detached', 'out-of-bounds']) {
    var source = new Uint8Array([1, 2]), observed = 0, proxyReads = 0;
    var target = new Uint8Array(2);
    if (kind === 'object') target = {};
    else if (kind === 'proxy') target = new Proxy(target, { get: function() { proxyReads++; throw 'brand hook'; } });
    else if (kind === 'detached') __lilaDetachArrayBuffer(target.buffer);
    else {
      var buffer = new ArrayBuffer(4, { maxByteLength: 8 });
      target = new Uint8Array(buffer, 2, 2); buffer.resize(1);
    }
    source.constructor = { [Symbol.species]: function() { observed++; return target; } };
    var error = capture(function() { invoke(method, source, name); }, kind);
    require(Object.getPrototypeOf(error) === errorPrototype && observed === 1 && proxyReads === 0, 'fresh result validation');
  }
  var source = new Uint8Array([1, 2]), short = new Uint8Array(0);
  source.constructor = { [Symbol.species]: function() { return short; } };
  if (name === 'subarray') require(invoke(method, source, name) === short, 'buffer-list result has no numeric minimum');
  else require(Object.getPrototypeOf(capture(function() { invoke(method, source, name); }, 'minimum')) === errorPrototype,
    'one-Number argument minimum');
  for (var bigintSource of [false, true]) {
    var source = bigintSource ? new BigInt64Array(0) : new Uint8Array(0);
    var target = bigintSource ? new Uint8Array(0) : new BigInt64Array(0);
    source.constructor = { [Symbol.species]: function() { return target; } };
    var error = capture(function() { invoke(method, source, name); }, 'content');
    require(Object.getPrototypeOf(error) === errorPrototype, 'empty content mismatch');
  }
  var source = new Float64Array([257.9, -1, 2.5]), target = new Uint8Array([99, 99, 99, 99, 99]);
  source.constructor = { [Symbol.species]: function() { return target; } };
  require(invoke(method, source, name) === target, 'returned view identity');
  require(target.length === 5 && target[3] === 99 && target[4] === 99, 'larger returned view');
  if (name === 'subarray') require(target[0] === 99 && target[1] === 99, 'subarray does not copy');
  else require(target[0] === 1 && target[1] === 255 && target[2] === 2, 'different Number kind conversion');
  var source = new BigInt64Array([-1n, 2n]), target = new BigUint64Array(2);
  source.constructor = { [Symbol.species]: function() { return target; } };
  require(invoke(method, source, name) === target, 'BigInt returned view');
  if (name !== 'subarray') require(target[0] === 18446744073709551615n && target[1] === 2n, 'same content BigInt conversion');
}

// Species can mutate the source before map/slice reads, after filter snapshots.
for (var name of ['map', 'filter', 'slice']) {
  var source = new Uint8Array([1, 2]);
  source.constructor = { [Symbol.species]: function(length) { source[0] = 9; return new Uint8Array(length); } };
  var result = invoke(Uint8Array.prototype[name], source, name);
  require(result[0] === (name === 'filter' ? 1 : 9), 'method source snapshot order');
}

// Slice retains bit encodings and the specified ascending overlapping copy.
for (var Constructor of [Float16Array, Float32Array, Float64Array]) {
  var width = Constructor.BYTES_PER_ELEMENT, buffer = new ArrayBuffer(width * 3);
  var bytes = new Uint8Array(buffer);
  for (var index = 0; index < bytes.length; index++) bytes[index] = (index * 37 + 1) & 255;
  if (width === 2) { var words = new Uint16Array(buffer); words[1] = 0x7c01; words[2] = 0xfe13; }
  else if (width === 4) { var words = new Uint32Array(buffer); words[1] = 0x7f800001; words[2] = 0xff801234; }
  else { var words = new Uint32Array(buffer); words[2] = 1; words[3] = 0x7ff00000; words[4] = 0x1234; words[5] = 0xfff00000; }
  var source = new Constructor(buffer, width, 2), target = new Constructor(2);
  source.constructor = { [Symbol.species]: function() { return target; } };
  require(source.slice(0, 2) === target, 'same-kind slice target');
  var copied = new Uint8Array(target.buffer);
  for (var index = 0; index < copied.length; index++) require(copied[index] === bytes[index + width], 'NaN bit encoding');
}
var source = new Uint8Array([1, 2, 3, 4]), target = new Uint8Array(source.buffer, 1, 3);
source.constructor = { [Symbol.species]: function() { return target; } };
require(source.slice(0, 3) === target && source[0] === 1 && source[1] === 1 && source[2] === 1 && source[3] === 1,
  'ascending overlap copy');

// Slice validates source again only if the captured count is nonzero.
for (var nonempty of [false, true]) {
  var source = new Uint8Array([1, 2]), target = new Uint8Array(nonempty ? 2 : 0);
  source.constructor = { [Symbol.species]: function() { __lilaDetachArrayBuffer(source.buffer); return target; } };
  if (nonempty) require(Object.getPrototypeOf(capture(function() { source.slice(0, 2); }, 'source detached')) === TypeError.prototype,
    'nonempty source revalidation');
  else require(source.slice(0, 0) === target, 'empty slice does not revalidate source');
}

// Subarray's exact two/three argument vectors preserve tracking versus fixed views.
for (var Constructor of [Uint16Array, BigInt64Array]) {
  var width = Constructor.BYTES_PER_ELEMENT, buffer = new ArrayBuffer(width * 5, { maxByteLength: width * 8 });
  var tracking = new Constructor(buffer, width), saved, calls = 0;
  tracking.constructor = { [Symbol.species]: function(actualBuffer, offset, length) {
    calls++; saved = arguments;
    return arguments.length === 2 ? new Constructor(actualBuffer, offset) : new Constructor(actualBuffer, offset, length);
  } };
  var result = tracking.subarray(1);
  require(saved.length === 2 && !Object.prototype.hasOwnProperty.call(saved, '2') &&
    saved[0] === buffer && saved[1] === width * 2 && result.length === 3, 'tracking argument vector');
  buffer.resize(width * 7); require(result.length === 5, 'tracking result growth');
  var undefinedEnd = tracking.subarray(1, undefined);
  require(saved.length === 2 && !Object.prototype.hasOwnProperty.call(saved, '2') &&
    saved[0] === buffer && saved[1] === width * 2 && undefinedEnd.length === 5, 'explicit undefined tracking vector');
  var fixed = tracking.subarray(1, 3);
  require(saved.length === 3 && saved[2] === 2 && fixed.length === 2, 'explicit end vector');
  buffer.resize(width * 8); require(fixed.length === 2, 'fixed result length');
  var fixedSource = new Constructor(buffer, width, 2);
  fixedSource.constructor = tracking.constructor;
  var fixedResult = fixedSource.subarray(1);
  require(saved.length === 3 && saved[2] === 1 && fixedResult.length === 1 && calls === 4, 'fixed source vector');
}

// An OOB/detached subarray source snapshots zero and still reaches custom species.
var buffer = new ArrayBuffer(4, { maxByteLength: 8 }), source = new Uint8Array(buffer, 2, 2);
buffer.resize(1);
var target = new Uint8Array(0), order = [];
source.constructor = { [Symbol.species]: function(actualBuffer, offset, length) {
  order.push('construct'); require(actualBuffer === buffer && offset === 2 && length === 0, 'stored OOB range'); return target;
} };
require(source.subarray({ valueOf: function() { order.push('start'); buffer.resize(4); return 1; } }, 1) === target &&
  order.join(',') === 'start,construct', 'subarray OOB snapshot');
var source = new Uint8Array(2), buffer = source.buffer;
__lilaDetachArrayBuffer(buffer);
source.constructor = { [Symbol.species]: function(actualBuffer, offset, length) {
  require(actualBuffer === buffer && offset === 0 && length === 0, 'detached snapshot'); return target;
} };
require(source.subarray(0, 0) === target, 'custom detached source species');
print('typed-array-species-views:ok');
262;
