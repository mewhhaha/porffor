const other = __lilaCreateRealm().global;
const kinds = [
  ['Int8Array', false], ['Uint8Array', false], ['Uint8ClampedArray', false],
  ['Int16Array', false], ['Uint16Array', false], ['Int32Array', false],
  ['Uint32Array', false], ['Float16Array', false], ['Float32Array', false],
  ['Float64Array', false], ['BigInt64Array', true], ['BigUint64Array', true]
];
const localBufferPrototype = ArrayBuffer.prototype;
const foreignBufferPrototype = other.ArrayBuffer.prototype;
const localMethods = [Int8Array.prototype.toReversed, Int8Array.prototype.toSorted, Int8Array.prototype.with];
const foreignMethods = [other.Int8Array.prototype.toReversed, other.Int8Array.prototype.toSorted, other.Int8Array.prototype.with];
const saved = [];
let forbiddenGets = 0;
function forbidden() { forbiddenGets++; throw 'constructor or species observed'; }
function replacementConstructor() { throw 'public constructor called'; }

for (let i = 0; i < kinds.length; i++) {
  const name = kinds[i][0];
  const local = globalThis[name];
  const foreign = other[name];
  saved.push([local, foreign, local.prototype, foreign.prototype, kinds[i][1]]);
  Object.defineProperty(local, Symbol.species, { configurable: true, get: forbidden });
  Object.defineProperty(foreign, Symbol.species, { configurable: true, get: forbidden });
  Object.defineProperty(local.prototype, 'constructor', { configurable: true, get: forbidden });
  Object.defineProperty(foreign.prototype, 'constructor', { configurable: true, get: forbidden });
  globalThis[name] = replacementConstructor;
  other[name] = replacementConstructor;
}
globalThis.ArrayBuffer = replacementConstructor;
other.ArrayBuffer = replacementConstructor;

let results = 0;
for (let i = 0; i < saved.length; i++) {
  const row = saved[i];
  const input = row[4] ? [3n, 1n, 2n] : [3, 1, 2];
  const changed = row[4] ? 9n : 9;
  const expected = [
    row[4] ? [2n, 1n, 3n] : [2, 1, 3],
    row[4] ? [1n, 2n, 3n] : [1, 2, 3],
    row[4] ? [3n, 9n, 2n] : [3, 9, 2]
  ];
  for (let direction = 0; direction < 2; direction++) {
    const Source = direction === 0 ? row[1] : row[0];
    const source = new Source(input);
    Object.defineProperty(source, 'constructor', { get: forbidden });
    const methods = direction === 0 ? localMethods : foreignMethods;
    const prototype = direction === 0 ? row[2] : row[3];
    const bufferPrototype = direction === 0 ? localBufferPrototype : foreignBufferPrototype;
    for (let method = 0; method < methods.length; method++) {
      const result = method === 2 ? methods[method].call(source, 1, changed) : methods[method].call(source);
      if (result === source || result.buffer === source.buffer || result.length !== 3) throw 'fresh same-type result';
      if (Object.getPrototypeOf(result) !== prototype || Object.getPrototypeOf(result.buffer) !== bufferPrototype) throw 'defining Realm';
      if (result.buffer.byteLength !== 3 * Source.BYTES_PER_ELEMENT) throw 'same element kind';
      for (let index = 0; index < 3; index++) {
        if (result[index] !== expected[method][index] || source[index] !== input[index]) throw 'same-type values';
      }
      results++;
    }
  }
}
if (results !== 72 || forbiddenGets !== 0) throw 'all kinds without constructor or species Get';
print('typed-array-same-type-realms:ok');
262;
