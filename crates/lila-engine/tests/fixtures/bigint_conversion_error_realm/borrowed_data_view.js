const foreign = __lilaCreateRealm().global;
const localType = TypeError.prototype;
const localSyntax = SyntaxError.prototype;
const foreignType = foreign.TypeError.prototype;
const foreignSyntax = foreign.SyntaxError.prototype;
const methods = [
  [DataView.prototype.setBigInt64, localType, localSyntax],
  [DataView.prototype.setBigUint64, localType, localSyntax],
  [foreign.DataView.prototype.setBigInt64, foreignType, foreignSyntax],
  [foreign.DataView.prototype.setBigUint64, foreignType, foreignSyntax]
];
foreign.TypeError = null;
foreign.SyntaxError = null;
function expectPrototype(operation, prototype) {
  let caught;
  try { operation(); } catch (error) { caught = error; }
  if (!caught || Object.getPrototypeOf(caught) !== prototype) throw 'wrong setter error prototype';
}
for (const [set, typePrototype, syntaxPrototype] of methods) {
  const view = new DataView(new ArrayBuffer(16));
  const trace = [];
  const endian = {[Symbol.toPrimitive]() { throw 'ToBoolean called a hook'; }};
  const offset = {[Symbol.toPrimitive](hint) { trace.push('offset:' + hint); return 15; }};
  const number = {[Symbol.toPrimitive](hint) {
    trace.push('value:' + hint);
    expectPrototype(() => BigInt(undefined), localType);
    return 1;
  }};
  expectPrototype(() => set.call(view, offset, number, endian), typePrototype);
  if (trace.join(',') !== 'offset:number,value:number') throw 'setter conversion order';
  for (const value of [undefined, null, Symbol('invalid')]) {
    expectPrototype(() => set.call(view, 0, value), typePrototype);
  }
  expectPrototype(() => set.call(view, 0, {
    [Symbol.toPrimitive](hint) {
      if (hint !== 'number') throw 'setter hint';
      return 'invalid';
    }
  }), syntaxPrototype);
  const marker = new TypeError('entry marker');
  let caught;
  try { set.call(view, 0, {valueOf() { throw marker; }}); }
  catch (error) { caught = error; }
  if (caught !== marker || Object.getPrototypeOf(caught) !== localType) throw 'setter abrupt identity';
  trace.length = 0;
  expectPrototype(() => set.call({}, offset, number, endian), typePrototype);
  if (trace.length !== 0) throw 'brand check observed conversions';
  set.call(view, 0, true, true);
  if (view.getBigUint64(0, true) !== 1n) throw 'Boolean setter success';
  set.call(view, 0, '0x10', false);
  if (view.getBigUint64(0, false) !== 16n) throw 'String setter success';
}
print('dataview:ok');
