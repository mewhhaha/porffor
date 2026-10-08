const trace = [];
let skippedCalls = 0;
function skipped() {
  skippedCalls++;
  Promise.resolve().then(() => trace.push('skipped-job'));
  throw 'skipped RHS evaluated';
}
async function run() {
  let zero = -0;
  let falseValue = false;
  let empty = '';
  let nan = NaN;
  let big = 0n;
  let object = {};
  let symbol = Symbol('saved');
  let htmlDda = __lilaCreateHTMLDDA();
  const originalObject = object;
  const originalSymbol = symbol;
  const originalHtml = htmlDda;
  if (!Object.is(zero &&= await skipped(), -0) || !Object.is(zero ??= await skipped(), -0)) throw 'signed zero skip';
  if ((falseValue &&= await skipped()) !== false || (falseValue ??= await skipped()) !== false) throw 'false skip';
  if ((empty &&= await skipped()) !== '' || (empty ??= await skipped()) !== '') throw 'empty string skip';
  if (!Object.is(nan &&= await skipped(), NaN) || !Object.is(nan ??= await skipped(), NaN)) throw 'NaN skip';
  if ((big &&= await skipped()) !== 0n || (big ??= await skipped()) !== 0n) throw 'BigInt zero skip';
  if ((object ||= await skipped()) !== originalObject || (object ??= await skipped()) !== originalObject) throw 'object identity skip';
  if ((symbol ||= await skipped()) !== originalSymbol || (symbol ??= await skipped()) !== originalSymbol) throw 'Symbol identity skip';
  if ((htmlDda &&= await skipped()) !== originalHtml || (htmlDda ??= await skipped()) !== originalHtml) throw 'HTMLDDA Boolean versus nullish selection';
  const constant = 3;
  if ((constant ||= await skipped()) !== 3 || skippedCalls !== 0) throw 'skipped const Put';
  let assigned = 0;
  const thenable = {get then() {
    trace.push('then'); assigned = 9;
    return resolve => { trace.push('resolve'); resolve(17); };
  }};
  function rhs() { trace.push('rhs'); assigned = 7; return thenable; }
  const result = assigned ||= await rhs();
  if (result !== 17 || assigned !== 17 || trace.join(',') !== 'rhs,then,caller,resolve') throw 'selected original binding and scheduling';
  let truthy = 1;
  if ((truthy &&= await 23) !== 23 || truthy !== 23) throw 'And selected assignment';
  let nullish = null;
  if ((nullish ??= await undefined) !== undefined || nullish !== undefined) throw 'undefined selected result';
  let nested = 0;
  const joined = nested ||= (false || (true ? await 31 : await skipped()));
  if (joined !== 31 || nested !== 31 || skippedCalls !== 0) throw 'nested branch result';
}
run().then(() => print('logical-assignment-values:ok'), error => print('unexpected:' + error));
trace.push('caller');
