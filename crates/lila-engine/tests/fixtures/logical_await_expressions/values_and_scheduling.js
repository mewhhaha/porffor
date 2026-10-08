const trace = [];
let rhsCalls = 0;
let skippedThenGets = 0;
let primitiveGets = 0;
let primitiveCalls = 0;
const symbol = Symbol('left');
const htmlDda = __lilaCreateHTMLDDA();
const object = {
  get [Symbol.toPrimitive]() {
    primitiveGets++;
    return () => { primitiveCalls++; throw 'unexpected primitive conversion'; };
  }
};
const skippedThenable = {get then() { skippedThenGets++; throw 'skipped then getter'; }};
function skipped() {
  rhsCalls++;
  Promise.resolve().then(() => trace.push('skipped-job'));
  return skippedThenable;
}
async function run() {
  const andMinusZero = -0 && await skipped();
  const andNaN = NaN && await skipped();
  const andEmpty = '' && await skipped();
  const andBigInt = 0n && await skipped();
  const andNull = null && await skipped();
  const andUndefined = undefined && await skipped();
  const orSymbol = symbol || await skipped();
  const orObject = object || await skipped();
  const orBigInt = 1n || await skipped();
  const nullishMinusZero = -0 ?? await skipped();
  const nullishNaN = NaN ?? await skipped();
  const nullishEmpty = '' ?? await skipped();
  const nullishBigInt = 0n ?? await skipped();
  const nullishFalse = false ?? await skipped();
  const nullishSymbol = symbol ?? await skipped();
  const nullishObject = object ?? await skipped();
  const kept = htmlDda ?? await skipped();
  const falsy = htmlDda && await skipped();
  if (!Object.is(andMinusZero, -0) || !Object.is(andNaN, NaN)) throw 'original falsy number';
  if (andEmpty !== '' || andBigInt !== 0n || andNull !== null || andUndefined !== undefined) throw 'original falsy value';
  if (orSymbol !== symbol || orObject !== object || orBigInt !== 1n) throw 'original truthy value';
  if (!Object.is(nullishMinusZero, -0) || !Object.is(nullishNaN, NaN)) throw 'nullish number predicate';
  if (nullishEmpty !== '' || nullishBigInt !== 0n || nullishFalse !== false) throw 'nullish falsy predicate';
  if (nullishSymbol !== symbol || nullishObject !== object) throw 'nullish identity';
  if (kept !== htmlDda || falsy !== htmlDda) throw 'HTMLDDA truthiness and nullish identity';
  let reads = 0;
  let andSource = -0;
  let orSource = symbol;
  let nullishSource = object;
  const cells = {
    get and() { reads++; const value = andSource; andSource = 1; return value; },
    get or() { reads++; const value = orSource; orSource = null; return value; },
    get nullish() { reads++; const value = nullishSource; nullishSource = undefined; return value; }
  };
  const savedAnd = cells.and && await skipped();
  const savedOr = cells.or || await skipped();
  const savedNullish = cells.nullish ?? await skipped();
  if (!Object.is(savedAnd, -0) || savedOr !== symbol || savedNullish !== object || reads !== 3) throw 'captured left GetValue';
  if (andSource !== 1 || orSource !== null || nullishSource !== undefined) throw 'left getter mutation';
  if (rhsCalls !== 0 || skippedThenGets !== 0 || primitiveGets !== 0 || primitiveCalls !== 0) throw 'skipped hooks';
  trace.push('skips');
  let selectedReads = 0;
  let selectedSource = object;
  const selectedCell = {get value() { selectedReads++; trace.push('left'); return selectedSource; }};
  function selected() {
    trace.push('rhs');
    selectedSource = 0;
    return {get then() {
      trace.push('then');
      return resolve => { trace.push('resolve'); resolve(17); };
    }};
  }
  const selectedAnd = selectedCell.value && await selected();
  trace.push('join');
  if (selectedAnd !== 17 || selectedReads !== 1 || selectedSource !== 0) throw 'selected left capture';
  if (trace.join(',') !== 'skips,left,rhs,then,caller,resolve,join') throw 'skip and selected scheduling';
  const fromNull = null ?? await 23;
  const fromUndefined = undefined ?? await 24;
  const fromFalse = false || await 25;
  const fromObject = object && await 26;
  if (fromNull !== 23 || fromUndefined !== 24 || fromFalse !== 25 || fromObject !== 26) throw 'selected logical values';
  if (rhsCalls !== 0 || skippedThenGets !== 0 || primitiveGets !== 0 || primitiveCalls !== 0) throw 'logical conversion hooks';
}
run().then(() => print('logical-values:ok'), error => print('unexpected:' + error));
trace.push('caller');
