function check(condition, name) { if (!condition) throw name; }
function poison() { throw 'canonical or replaced String method'; }
function noString() { throw 'raw receiver must reach symbol hook'; }
const symbolResult = Symbol('String hook result');
function functionResult() { return 41; }
const values = [37, functionResult, symbolResult];
const splitReceiver = {saved: String.prototype.split, split: poison, toString: noString};
const matchAllReceiver = {saved: String.prototype.matchAll, matchAll: poison, toString: noString};
const originalLimit = {valueOf() { throw 'hook limit coerced'; }};
let splitGets = 0, splitCalls = 0, matchAllGets = 0, matchAllCalls = 0;
for (let index = 0; index < values.length; index++) {
  const result = values[index];
  const splitPattern = {};
  Object.defineProperty(splitPattern, Symbol.split, {get() {
    splitGets++; check(this === splitPattern, 'split getter this');
    return new Proxy(function() {}, {apply(target, receiver, args) {
      splitCalls++; check(receiver === splitPattern && args.length === 2 && args[0] === splitReceiver && args[1] === originalLimit,
        'split hook raw Reference'); return result;
    }});
  }});
  const splitValue = splitReceiver.saved(splitPattern, originalLimit, 'ignored');
  check(splitValue === result, 'split arbitrary hook result');
  const allPattern = {[Symbol.match]: false};
  Object.defineProperty(allPattern, Symbol.matchAll, {get() {
    matchAllGets++; check(this === allPattern, 'matchAll getter this');
    return new Proxy(function() {}, {apply(target, receiver, args) {
      matchAllCalls++; check(receiver === allPattern && args.length === 1 && args[0] === matchAllReceiver,
        'matchAll hook raw Reference'); return result;
    }});
  }});
  const allValue = matchAllReceiver.saved(...[allPattern], originalLimit, 'ignored');
  check(allValue === result, 'matchAll spread arbitrary hook result');
  if (index === 0) check(splitValue + 1 === 38 && allValue + 1 === 38, 'hook Number result stays numeric');
  if (index === 1) check(splitValue() === 41 && allValue() === 41, 'hook Function result remains callable');
  if (index === 2) check(typeof splitValue === 'symbol' && typeof allValue === 'symbol', 'hook Symbol result keeps tag');
}
check(splitGets === 3 && splitCalls === 3 && matchAllGets === 3 && matchAllCalls === 3, 'single original Get and Call');

const matchReceiver = {saved: String.prototype.match, match: poison, toString: noString};
const matchPattern = {[Symbol.match](raw) {
  check(this === matchPattern && raw === matchReceiver && arguments.length === 1, 'match hook role'); return 43;
}};
check(matchReceiver.saved(matchPattern, 'ignored') === 43, 'match arbitrary Number');
const replaceReceiver = {saved: String.prototype.replace, replace: poison, toString: noString};
const replacement = {toString: poison};
const replacePattern = {[Symbol.replace](raw, second) {
  check(this === replacePattern && raw === replaceReceiver && second === replacement && arguments.length === 2,
    'replace hook role'); return symbolResult;
}};
check(replaceReceiver.saved(replacePattern, replacement, 'ignored') === symbolResult, 'replace arbitrary Symbol');
const replaceAllReceiver = {saved: String.prototype.replaceAll, replaceAll: poison, toString: noString};
const replaceAllPattern = {[Symbol.match]: false, [Symbol.replace](raw, second) {
  check(this === replaceAllPattern && raw === replaceAllReceiver && second === replacement && arguments.length === 2,
    'replaceAll hook role'); return functionResult;
}};
check(replaceAllReceiver.saved(replaceAllPattern, replacement)() === 41, 'replaceAll arbitrary Function');
const searchReceiver = {saved: String.prototype.search, search: poison, toString: noString};
const searchPattern = {[Symbol.search](raw) {
  check(this === searchPattern && raw === searchReceiver && arguments.length === 1, 'search hook role'); return symbolResult;
}};
check(searchReceiver.saved(searchPattern, 'ignored') === symbolResult, 'search arbitrary Symbol');

let receiverFlow = 1;
const concatTrace = [];
const concatReceiver = {saved: String.prototype.concat, concat: poison, toString() {
  concatTrace.push('receiver'); receiverFlow = function() { return 47; }; return 'a';
}};
const left = {toString() { concatTrace.push('left'); return 'b'; }};
const right = {toString() { concatTrace.push('right'); return 'c'; }};
check(concatReceiver.saved((concatTrace.push('left.arg'), left), (concatTrace.push('right.arg'), right)) === 'abc' &&
  typeof receiverFlow === 'function' && receiverFlow() === 47 &&
  concatTrace.join(',') === 'left.arg,right.arg,receiver,left,right', 'concat receiver/argument coercion effects');
const indexFlow = {value: 1};
const indexed = {saved: String.prototype.at, at: poison, toString() { return 'abc'; }};
const index = {valueOf() { indexFlow.value = 'index-hook'; return -1; }};
check(indexed.saved(index) === 'c' && typeof indexFlow.value === 'string' && indexFlow.value.charAt(0) === 'i',
  'numeric hook invalidates captured shape');
let paddingFlow = 1;
const padded = {saved: String.prototype.padStart, padStart: poison, toString() { return 'x'; }};
const filler = {toString() { paddingFlow = symbolResult; return '0'; }};
check(padded.saved(3, filler) === '00x' && typeof paddingFlow === 'symbol' && paddingFlow === symbolResult,
  'padding String coercion invalidates captured kind');
const normalizationFlow = {value: 1};
const normalized = {saved: String.prototype.normalize, normalize: poison, toString() { return 'e\u0301'; }};
const form = {toString() { normalizationFlow.value = functionResult; return 'NFC'; }};
check(normalized.saved(form) === '\u00E9' && typeof normalizationFlow.value === 'function' && normalizationFlow.value() === 41,
  'normalization form coercion invalidates shape');
const hookFlow = [1];
const effectReceiver = {saved: String.prototype.matchAll, matchAll: poison, toString: noString};
const effectPattern = {[Symbol.match]: false};
Object.defineProperty(effectPattern, Symbol.matchAll, {get() {
  hookFlow[0] = functionResult;
  return new Proxy(function() {}, {apply() { return symbolResult; }});
}});
check(effectReceiver.saved(effectPattern) === symbolResult && typeof hookFlow[0] === 'function' && hookFlow[0]() === 41,
  'symbol getter invalidates captured element facts');
print('string-invocation-hooks:ok');
262;
