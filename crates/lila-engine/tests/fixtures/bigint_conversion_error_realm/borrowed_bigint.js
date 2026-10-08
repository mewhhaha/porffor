const foreign = __lilaCreateRealm().global;
const localType = TypeError.prototype;
const localSyntax = SyntaxError.prototype;
const foreignType = foreign.TypeError.prototype;
const foreignSyntax = foreign.SyntaxError.prototype;
const foreignRange = foreign.RangeError.prototype;
const convert = foreign.BigInt;
foreign.TypeError = null;
foreign.SyntaxError = null;
foreign.RangeError = null;
foreign.BigInt = null;
function expectPrototype(operation, prototype) {
  let caught;
  try { operation(); } catch (error) { caught = error; }
  if (!caught || Object.getPrototypeOf(caught) !== prototype) throw 'wrong error prototype';
}
for (const value of [undefined, null, Symbol('invalid')]) {
  expectPrototype(() => convert(value), foreignType);
  expectPrototype(() => BigInt(value), localType);
}
for (const value of ['invalid', '1.5', '1n']) {
  expectPrototype(() => convert(value), foreignSyntax);
  expectPrototype(() => BigInt(value), localSyntax);
}
for (const value of [NaN, Infinity, -Infinity, 1.5]) {
  expectPrototype(() => convert(value), foreignRange);
}
const trace = [];
expectPrototype(() => convert({[Symbol.toPrimitive](hint) {
  trace.push('foreign:' + hint);
  expectPrototype(() => BigInt(undefined), localType);
  return 'invalid';
}}), foreignSyntax);
expectPrototype(() => convert({valueOf() {
  trace.push('valueOf'); return Symbol('invalid');
}, toString() { throw 'late toString'; }}), foreignType);
if (trace.join(',') !== 'foreign:number,valueOf') throw 'conversion order';
const marker = new TypeError('entry marker');
let caught;
try { convert({[Symbol.toPrimitive]() { throw marker; }}); }
catch (error) { caught = error; }
if (caught !== marker || Object.getPrototypeOf(caught) !== localType) throw 'changed abrupt value';
if (convert(true) !== 1n || convert(false) !== 0n || convert(' 0x10 ') !== 16n ||
    convert(9) !== 9n || convert(11n) !== 11n) throw 'successful conversion';
function lexicalWriter(index) {
  const target = new BigInt64Array(1);
  return value => { target[index] = value; };
}
const write = lexicalWriter(0);
expectPrototype(() => write(1), localType);
expectPrototype(() => write('invalid'), localSyntax);
print('bigint:ok');
