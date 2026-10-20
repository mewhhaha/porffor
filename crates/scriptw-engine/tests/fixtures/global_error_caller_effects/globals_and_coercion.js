function check(condition, name) { if (!condition) throw name; }
function poison() { throw 'replaced global or canonical property'; }
const define = Object.defineProperty;
const descriptor = Object.getOwnPropertyDescriptor;
const savedEncodeURI = encodeURI;
const savedEncodeComponent = encodeURIComponent;
const encodeDescriptor = descriptor(globalThis, 'encodeURI');
function restore(target, key, original) {
  if (original === undefined) delete target[key];
  else define(target, key, original);
}

// Each literal call keeps a distinct native target rather than merging a method table.
function escapeEffects() {
  let kind = 1, shape = {value: 1}, elements = [1];
  const input = {toString() {
    kind = function() { return 7; }; shape = {value: Symbol.for('escape effect')};
    elements[0] = function() { return 9; }; return 'A B';
  }};
  const result = escape(input);
  check(result === 'A%20B' && typeof kind === 'function' && kind() === 7 &&
    typeof shape.value === 'symbol' && typeof elements[0] === 'function' && elements[0]() === 9,
    'escape ToString invalidates captured kind, shape and element facts');
}
function encodeURIEffects() {
  let kind = 1, shape = {value: 1}, elements = [1];
  const input = {toString() {
    kind = function() { return 7; }; shape = {value: Symbol.for('encode URI effect')};
    elements[0] = function() { return 9; }; return 'A B';
  }};
  const result = encodeURI(input);
  check(result === 'A%20B' && typeof kind === 'function' && kind() === 7 &&
    typeof shape.value === 'symbol' && typeof elements[0] === 'function' && elements[0]() === 9,
    'encodeURI ToString invalidates captured kind, shape and element facts');
}
function encodeComponentEffects() {
  let kind = 1, shape = {value: 1}, elements = [1];
  const input = {toString() {
    kind = function() { return 7; }; shape = {value: Symbol.for('encode component effect')};
    elements[0] = function() { return 9; }; return 'A B';
  }};
  const result = encodeURIComponent(input);
  check(result === 'A%20B' && typeof kind === 'function' && kind() === 7 &&
    typeof shape.value === 'symbol' && typeof elements[0] === 'function' && elements[0]() === 9,
    'encodeURIComponent ToString invalidates captured kind, shape and element facts');
}
function decodeURIEffects() {
  let kind = 1, shape = {value: 1}, elements = [1];
  const input = {toString() {
    kind = function() { return 7; }; shape = {value: Symbol.for('decode URI effect')};
    elements[0] = function() { return 9; }; return 'A%20B';
  }};
  const result = decodeURI(input);
  check(result === 'A B' && typeof kind === 'function' && kind() === 7 &&
    typeof shape.value === 'symbol' && typeof elements[0] === 'function' && elements[0]() === 9,
    'decodeURI ToString invalidates captured kind, shape and element facts');
}
function decodeComponentEffects() {
  let kind = 1, shape = {value: 1}, elements = [1];
  const input = {toString() {
    kind = function() { return 7; }; shape = {value: Symbol.for('decode component effect')};
    elements[0] = function() { return 9; }; return 'A%20B';
  }};
  const result = decodeURIComponent(input);
  check(result === 'A B' && typeof kind === 'function' && kind() === 7 &&
    typeof shape.value === 'symbol' && typeof elements[0] === 'function' && elements[0]() === 9,
    'decodeURIComponent ToString invalidates captured kind, shape and element facts');
}
function unescapeEffects() {
  let kind = 1, shape = {value: 1}, elements = [1];
  const input = {toString() {
    kind = function() { return 7; }; shape = {value: Symbol.for('unescape effect')};
    elements[0] = function() { return 9; }; return 'A%20B';
  }};
  const result = unescape(input);
  check(result === 'A B' && typeof kind === 'function' && kind() === 7 &&
    typeof shape.value === 'symbol' && typeof elements[0] === 'function' && elements[0]() === 9,
    'unescape keeps its corrected ToString caller effect');
}
escapeEffects(); encodeURIEffects(); encodeComponentEffects();
decodeURIEffects(); decodeComponentEffects(); unescapeEffects();

try {
  const directTrace = [];
  const directInput = {toString() { directTrace.push('coerce'); return 'A B'; }};
  function replaceDirect() { directTrace.push('extra'); globalThis.encodeURI = poison; return 9; }
  const direct = encodeURI((directTrace.push('first'), directInput), replaceDirect());
  check(direct === 'A%20B' && directTrace.join(',') === 'first,extra,coerce',
    'global callee is acquired before first and ignored operand effects');
  restore(globalThis, 'encodeURI', encodeDescriptor);

  const trace = [];
  const receiver = {encodeURIComponent: poison};
  const first = {toString() { trace.push('coerce'); return 'A B'; }};
  const proxy = new Proxy(savedEncodeComponent, {apply(target, rawThis, args) {
    trace.push('apply');
    check(rawThis === receiver && args.length === 4 && args[0] === first && args[1] === 17 &&
      args[2] === 23 && args[3] === 29, 'proxy observes original raw receiver and full operands');
    return Reflect.apply(target, rawThis, args);
  }});
  define(receiver, 'saved', {configurable: true, get() { trace.push('callee'); return proxy; }});
  function argument() {
    trace.push('first'); define(receiver, 'saved', {value: poison, configurable: true}); return first;
  }
  const result = receiver.saved(argument(), (trace.push('extra1'), 17),
    (trace.push('extra2'), 23), (trace.push('extra3'), 29));
  check(result === 'A%20B' && trace.join(',') === 'callee,first,extra1,extra2,extra3,apply,coerce',
    'transferred codec retains acquired callee and ignored extras');

  const spreadTrace = [];
  const spreadReceiver = {encodeURI: poison};
  const spreadFirst = {toString() { spreadTrace.push('coerce'); return 'A B'; }};
  spreadReceiver.saved = new Proxy(savedEncodeURI, {apply(target, rawThis, args) {
    spreadTrace.push('apply');
    check(rawThis === spreadReceiver && args.length === 3 && args[0] === spreadFirst &&
      args[1] === 31 && args[2] === 37, 'spread keeps full codec argv and raw this');
    return Reflect.apply(target, rawThis, args);
  }});
  let position = 0, closes = 0;
  const iterator = {};
  const next = new Proxy(function() {}, {apply(target, rawThis, args) {
    check(rawThis === iterator && args.length === 0, 'spread caches next and calls with no arguments');
    spreadTrace.push('next' + position);
    iterator.next = poison;
    if (position++ === 0) return {done: false, value: spreadFirst};
    if (position === 2) return {done: false, value: 31};
    return {done: true, get value() { throw 'terminal value must not be read'; }};
  }});
  define(iterator, 'next', {configurable: true, get() { spreadTrace.push('next.get'); return next; },
    set(value) { define(iterator, 'next', {value, writable: true, configurable: true}); }});
  define(iterator, 'return', {get() { closes++; throw 'spread has no close'; }});
  const source = {get [Symbol.iterator]() {
    spreadTrace.push('iterator.get'); return function() { spreadTrace.push('iterator.call'); return iterator; };
  }};
  const spreadResult = spreadReceiver.saved(...source, (spreadTrace.push('extra'), 37));
  check(spreadResult === 'A%20B' && closes === 0 &&
    spreadTrace.join(',') === 'iterator.get,iterator.call,next.get,next0,next1,next2,extra,apply,coerce',
    'real spread finishes before later ignored operand and native coercion');
} finally {
  restore(globalThis, 'encodeURI', encodeDescriptor);
}
print('global-error-globals:ok');
262;
