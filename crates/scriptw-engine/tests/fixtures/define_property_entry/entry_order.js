const methods = [Object.defineProperty, Reflect.defineProperty];
const getPrototypeOf = Object.getPrototypeOf;
const getOwnPropertyDescriptor = Object.getOwnPropertyDescriptor;
const typeErrorPrototype = TypeError.prototype;
const marker = {};
const primitives = [undefined, null, false, true, 0, -0, NaN, Infinity, '', 'primitive', 17n, Symbol('target')];
let invalidCases = 0;

function expectTypeError(action) {
  let returned = 'before';
  try {returned = action();} catch (error) {
    if (getPrototypeOf(error) !== typeErrorPrototype || returned !== 'before') throw 'entry target TypeError';
    return;
  }
  throw 'missing entry target TypeError';
}

for (const method of methods) {
  for (const target of primitives) {
    const trace = [];
    const key = {
      get [Symbol.toPrimitive]() {
        trace.push('keyGet');
        throw marker;
      }
    };
    const attributes = new Proxy({}, {
      has() {trace.push('attributesHas'); throw marker;},
      get() {trace.push('attributesGet'); throw marker;}
    });
    expectTypeError(() => method(
      (trace.push('targetArg'), target),
      (trace.push('keyArg'), key),
      (trace.push('attributesArg'), attributes)
    ));
    if (trace.join(',') !== 'targetArg,keyArg,attributesArg') throw 'primitive target observed conversion';
    invalidCases++;
  }
}
if (invalidCases !== 24) throw 'primitive target cohort';

// All caller arguments are evaluated even when the target check will fail.
for (const method of methods) {
  const trace = [];
  try {
    method((trace.push('targetArg'), 0), (trace.push('keyArg'), 'x'), (trace.push('attributesArg'), (() => {throw marker;})()));
    throw 'missing caller argument abrupt';
  } catch (error) {
    if (error !== marker || trace.join(',') !== 'targetArg,keyArg,attributesArg') throw 'caller argument evaluation precedence';
  }
}

for (let kind = 0; kind < methods.length; kind++) {
  const method = methods[kind];
  const trace = [];
  const target = {};
  const key = {
    get [Symbol.toPrimitive]() {
      trace.push('keyGet');
      return function (hint) {
        if (this !== key || hint !== 'string') throw 'definition key receiver or hint';
        trace.push('keyCall');
        return 'x';
      };
    }
  };
  const attributes = new Proxy({value: 9}, {
    has(_target, field) {trace.push('has:' + field); return field === 'value';},
    get(_target, field) {
      trace.push('get:' + field);
      if (field !== 'value') throw 'absent descriptor field Get';
      return 9;
    }
  });
  const result = method(
    (trace.push('targetArg'), target),
    (trace.push('keyArg'), key),
    (trace.push('attributesArg'), attributes)
  );
  if (result !== (kind === 0 ? target : true)) throw 'normal definition result policy';
  if (trace.join(',') !== 'targetArg,keyArg,attributesArg,keyGet,keyCall,has:enumerable,has:configurable,has:value,get:value,has:writable,has:get,has:set') throw 'valid target conversion order';
  const descriptor = getOwnPropertyDescriptor(target, 'x');
  if (descriptor.value !== 9 || descriptor.writable || descriptor.enumerable || descriptor.configurable) throw 'partial definition presence';

  const throwingKey = {[Symbol.toPrimitive]() {throw marker;}};
  let reads = 0;
  const untouchedAttributes = new Proxy({}, {has() {reads++; throw 'late descriptor hook';}});
  try {method({}, throwingKey, untouchedAttributes); throw 'missing key abrupt';} catch (error) {
    if (error !== marker || reads !== 0) throw 'key abrupt before descriptor';
  }
  const throwingAttributes = new Proxy({}, {has() {throw marker;}});
  try {method({}, 'x', throwingAttributes); throw 'missing descriptor abrupt';} catch (error) {
    if (error !== marker) throw 'original descriptor abrupt';
  }
}

print('define-property-entry-order:ok');
262;
