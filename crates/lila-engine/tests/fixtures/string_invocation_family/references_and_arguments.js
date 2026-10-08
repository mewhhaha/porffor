function check(condition, name) { if (!condition) throw name; }
function poison() { throw 'canonical or replaced String method'; }
let literalCases = 0;
const case0 = { saved: String.prototype.substr, substr: poison, toString() { return 'abcdef'; } };
check(case0.saved(-3, 2) === 'de', 'substr acquired alias');
literalCases++;
const case1 = { saved: String.prototype.charAt, charAt: poison, toString() { return 'abc'; } };
check(case1.saved(1) === 'b', 'charAt acquired alias');
literalCases++;
const case2 = { saved: String.prototype.charCodeAt, charCodeAt: poison, toString() { return 'abc'; } };
check(case2.saved(1) === 98, 'charCodeAt acquired alias');
literalCases++;
const case3 = { saved: String.prototype.codePointAt, codePointAt: poison, toString() { return 'A\uD83D\uDE00B'; } };
check(case3.saved(1) === 128512, 'codePointAt acquired alias');
literalCases++;
const case4 = { saved: String.prototype.at, at: poison, toString() { return 'abc'; } };
check(case4.saved(-1) === 'c', 'at acquired alias');
literalCases++;
const case5 = { saved: String.prototype.padStart, padStart: poison, toString() { return 'x'; } };
check(case5.saved(3, '0') === '00x', 'padStart acquired alias');
literalCases++;
const case6 = { saved: String.prototype.padEnd, padEnd: poison, toString() { return 'x'; } };
check(case6.saved(3, '0') === 'x00', 'padEnd acquired alias');
literalCases++;
const case7 = { saved: String.prototype.repeat, repeat: poison, toString() { return 'ab'; } };
check(case7.saved(3) === 'ababab', 'repeat acquired alias');
literalCases++;
const case8 = { saved: String.prototype.normalize, normalize: poison, toString() { return 'e\u0301'; } };
check(case8.saved('NFC') === '\u00E9', 'normalize acquired alias');
literalCases++;
const case9 = { saved: String.prototype.localeCompare, localeCompare: poison, toString() { return 'a'; } };
check(case9.saved('b') < 0, 'localeCompare acquired alias');
literalCases++;
const case10 = { saved: String.prototype.toLocaleLowerCase, toLocaleLowerCase: poison, toString() { return 'ABC'; } };
check(case10.saved() === 'abc', 'toLocaleLowerCase acquired alias');
literalCases++;
const case11 = { saved: String.prototype.toLocaleUpperCase, toLocaleUpperCase: poison, toString() { return 'abc'; } };
check(case11.saved() === 'ABC', 'toLocaleUpperCase acquired alias');
literalCases++;
const case12 = { saved: String.prototype.toLowerCase, toLowerCase: poison, toString() { return 'ABC'; } };
check(case12.saved() === 'abc', 'toLowerCase acquired alias');
literalCases++;
const case13 = { saved: String.prototype.toUpperCase, toUpperCase: poison, toString() { return 'abc'; } };
check(case13.saved() === 'ABC', 'toUpperCase acquired alias');
literalCases++;
const case14 = { saved: String.prototype.isWellFormed, isWellFormed: poison, toString() { return 'A\uD800'; } };
check(case14.saved() === false, 'isWellFormed acquired alias');
literalCases++;
const case15 = { saved: String.prototype.toWellFormed, toWellFormed: poison, toString() { return 'A\uD800'; } };
check(case15.saved() === 'A\uFFFD', 'toWellFormed acquired alias');
literalCases++;
const case16 = { saved: String.prototype.anchor, anchor: poison, toString() { return 'x'; } };
check(case16.saved('value"x') === '<a name="value&quot;x">x</a>', 'anchor acquired alias');
literalCases++;
const case17 = { saved: String.prototype.big, big: poison, toString() { return 'x'; } };
check(case17.saved() === '<big>x</big>', 'big acquired alias');
literalCases++;
const case18 = { saved: String.prototype.blink, blink: poison, toString() { return 'x'; } };
check(case18.saved() === '<blink>x</blink>', 'blink acquired alias');
literalCases++;
const case19 = { saved: String.prototype.bold, bold: poison, toString() { return 'x'; } };
check(case19.saved() === '<b>x</b>', 'bold acquired alias');
literalCases++;
const case20 = { saved: String.prototype.fixed, fixed: poison, toString() { return 'x'; } };
check(case20.saved() === '<tt>x</tt>', 'fixed acquired alias');
literalCases++;
const case21 = { saved: String.prototype.fontcolor, fontcolor: poison, toString() { return 'x'; } };
check(case21.saved('red') === '<font color="red">x</font>', 'fontcolor acquired alias');
literalCases++;
const case22 = { saved: String.prototype.fontsize, fontsize: poison, toString() { return 'x'; } };
check(case22.saved(3) === '<font size="3">x</font>', 'fontsize acquired alias');
literalCases++;
const case23 = { saved: String.prototype.italics, italics: poison, toString() { return 'x'; } };
check(case23.saved() === '<i>x</i>', 'italics acquired alias');
literalCases++;
const case24 = { saved: String.prototype.link, link: poison, toString() { return 'x'; } };
check(case24.saved('/path') === '<a href="/path">x</a>', 'link acquired alias');
literalCases++;
const case25 = { saved: String.prototype.small, small: poison, toString() { return 'x'; } };
check(case25.saved() === '<small>x</small>', 'small acquired alias');
literalCases++;
const case26 = { saved: String.prototype.strike, strike: poison, toString() { return 'x'; } };
check(case26.saved() === '<strike>x</strike>', 'strike acquired alias');
literalCases++;
const case27 = { saved: String.prototype.sub, sub: poison, toString() { return 'x'; } };
check(case27.saved() === '<sub>x</sub>', 'sub acquired alias');
literalCases++;
const case28 = { saved: String.prototype.sup, sup: poison, toString() { return 'x'; } };
check(case28.saved() === '<sup>x</sup>', 'sup acquired alias');
literalCases++;
const case29 = { saved: String.prototype.trim, trim: poison, toString() { return ' \tfoo \n'; } };
check(case29.saved() === 'foo', 'trim acquired alias');
literalCases++;
const case30 = { saved: String.prototype.trimStart, trimStart: poison, toString() { return ' \tfoo \n'; } };
check(case30.saved() === 'foo \n', 'trimStart acquired alias');
literalCases++;
const case31 = { saved: String.prototype.trimEnd, trimEnd: poison, toString() { return ' \tfoo \n'; } };
check(case31.saved() === ' \tfoo', 'trimEnd acquired alias');
literalCases++;
const case32 = { saved: String.prototype.split, split: poison, toString() { return 'a:b:c'; } };
const split = case32.saved(':', 2);
check(Array.isArray(split) && split.length === 2 && split[0] === 'a' && split[1] === 'b', 'split acquired alias');
literalCases++;
check(literalCases === 33, 'all literal String family targets');

const nativeSubstr = String.prototype.substr;
const define = Object.defineProperty;
const trace = [];
const source = {substr: poison, toString() {
  check(this === source, 'raw substr receiver'); trace.push('receiver'); return 'abcdef';
}};
let selected = source;
define(source, 'cut', {configurable: true, get() { trace.push('callee'); return nativeSubstr; }});
function base() { trace.push('base'); return selected; }
function key() { trace.push('key'); return 'cut'; }
function startArgument() {
  trace.push('start.arg'); selected = {toString: poison};
  define(source, 'cut', {value: poison});
  return {valueOf() { trace.push('start.coerce'); return 1; }};
}
const count = {valueOf() { trace.push('count.coerce'); return 2; }};
let position = 0;
const iterator = {};
const next = new Proxy(function() {}, {apply(target, receiver, args) {
  check(receiver === iterator && args.length === 0, 'argument next Reference');
  const index = position++; check(index < 2, 'bounded spread'); trace.push('next' + index);
  if (index === 0) define(iterator, 'next', {value: poison});
  return {get done() { trace.push('done' + index); return index === 1; }, get value() {
    check(index === 0, 'done before value'); trace.push('value0'); return count;
  }};
}});
define(iterator, 'next', {configurable: true, get() { trace.push('next.get'); return next; }});
define(iterator, 'return', {get() { throw 'normal argument spread close'; }});
const spread = [99];
define(spread, '0', {get() { throw 'argument numeric snapshot'; }});
define(spread, Symbol.iterator, {get() {
  trace.push('iterator.get');
  return new Proxy(function() {}, {apply(target, receiver, args) {
    check(receiver === spread && args.length === 0, 'argument iterator Reference');
    trace.push('open'); return iterator;
  }});
}});
function extra(label) { trace.push(label); return {valueOf() { throw 'ignored extra coerced'; }}; }
check(base()[key()](startArgument(), ...spread, extra('tail'), extra('last')) === 'bc' && selected !== source,
  'acquired substr callee and original receiver survive all arguments');
check(trace.join(',') === 'base,key,callee,start.arg,iterator.get,open,next.get,next0,done0,value0,next1,done1,' +
  'tail,last,receiver,start.coerce,count.coerce', 'full spread and extras precede coercion');

const nativeRepeat = String.prototype.repeat;
const proxyTrace = [];
const proxyReceiver = {repeat: poison, toString() { proxyTrace.push('receiver'); return 'ab'; }};
proxyReceiver.saved = new Proxy(nativeRepeat, {apply(target, receiver, args) {
  check(target === nativeRepeat && receiver === proxyReceiver && args.length === 3 && args[1] === 'ignored' && args[2] === 42,
    'callable String Proxy receives complete raw arguments');
  proxyTrace.push('apply'); return Reflect.apply(target, receiver, args);
}});
check(proxyReceiver.saved((proxyTrace.push('count.arg'), 2), (proxyTrace.push('extra.arg'), 'ignored'),
  (proxyTrace.push('last.arg'), 42)) === 'abab' &&
  proxyTrace.join(',') === 'count.arg,extra.arg,last.arg,apply,receiver', 'Proxy forwarding and ignored operands');

const nativeCharCodeAt = String.prototype.charCodeAt;
const primitiveTrace = [];
let primitiveNext = 0;
const primitiveSpread = [99];
define(primitiveSpread, '0', {get() { throw 'primitive spread numeric snapshot'; }});
define(primitiveSpread, Symbol.iterator, {value() {
  primitiveTrace.push('open'); String.prototype.charCodeAt = poison;
  return {next() { primitiveTrace.push('next');
    return primitiveNext++ === 0 ? {value: 1, done: false} : {done: true}; }};
}});
try {
  const unit = 'abc'.charCodeAt(...primitiveSpread, (primitiveTrace.push('extra'), 42));
  check(unit === 98 && unit + 1 === 99 && primitiveTrace.join(',') === 'open,next,next,extra',
    'primitive charCodeAt retains acquired callee before genuine spread mutation');
} finally { String.prototype.charCodeAt = nativeCharCodeAt; }
check('abc'.charCodeAt(...[1]) + 1 === 99 && Number.isNaN('abc'.charCodeAt(...[99])),
  'charCodeAt spread has numeric result');
check('\uD83D\uDE00'.codePointAt(...[0]) + 1 === 128513 &&
  'abc'.codePointAt(...[99]) === undefined, 'codePointAt spread has Number or Undefined result');
print('string-invocation-references:ok');
262;
