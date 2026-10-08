const foreign = __lilaCreateRealm().global;
const LocalBigInt = BigInt64Array;
const ForeignBigInt = foreign.BigInt64Array;
const methods = [Array.prototype.sort, foreign.Array.prototype.sort];
const sources = [ForeignBigInt, LocalBigInt];

function makeSource(Constructor) {
  const source = new Constructor([3n, 1n]);
  Object.defineProperty(source, 'length', { value: 4, configurable: true });
  if (Reflect.has(source, '2') || Reflect.has(source, '3')) throw 'invalid indices must be absent';
  return source;
}
function checkIdentity(source, buffer, prototype, result) {
  if (result !== source || result.buffer !== buffer || Object.getPrototypeOf(result) !== prototype) throw 'sort receiver identity';
  if (source.length !== 4 || Object.getOwnPropertyDescriptor(source, 'length').value !== 4 ||
      source.byteLength !== 16 || buffer.byteLength !== 16) throw 'ordinary length and actual buffer';
  if (source[2] !== undefined || source[3] !== undefined || Reflect.has(source, '2') || Reflect.has(source, '3')) throw 'trailing absent indices';
}

let sortedCases = 0;
for (let direction = 0; direction < 2; direction++) {
  const Constructor = sources[direction];
  for (let custom = 0; custom < 2; custom++) {
    const source = makeSource(Constructor);
    const buffer = source.buffer;
    const prototype = Object.getPrototypeOf(source);
    let comparisons = 0;
    const compare = function(left, right) {
      comparisons++;
      if (typeof left !== 'bigint' || typeof right !== 'bigint') throw 'absent index entered comparison';
      return left < right ? -1 : left > right ? 1 : 0;
    };
    const result = custom ? methods[direction].call(source, compare) : methods[direction].call(source);
    checkIdentity(source, buffer, prototype, result);
    if (source[0] !== 1n || source[1] !== 3n || (custom && comparisons === 0)) throw 'sort actual two BigInts';
    sortedCases++;
  }
}
if (sortedCases !== 4) throw 'both comparison modes and borrowing directions';

const marker = new foreign.Error('foreign Array.sort comparator marker');
const abruptSource = makeSource(LocalBigInt);
const abruptBuffer = abruptSource.buffer;
const abruptPrototype = Object.getPrototypeOf(abruptSource);
const trace = [];
let assigned = 'prior';
try {
  assigned = methods[1].call(abruptSource, function(left, right) {
    if (typeof left !== 'bigint' || typeof right !== 'bigint') throw 'wrong abrupt comparison operands';
    trace.push('compare');
    throw marker;
  });
} catch (error) {
  if (error !== marker) throw 'lost foreign comparator identity';
  trace.push('catch');
} finally { trace.push('finally'); }
if (assigned !== 'prior' || trace.join() !== 'compare,catch,finally' ||
    abruptSource[0] !== 3n || abruptSource[1] !== 1n) throw 'abrupt before writeback';
checkIdentity(abruptSource, abruptBuffer, abruptPrototype, abruptSource);
print('array-sort-typed-array-presence:ok');
262;
