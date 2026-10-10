const intrinsicTypePrototype = TypeError.prototype;
TypeError = undefined;
const strictMode = (function() { return this === undefined; })();
const whole = {marker: 67};
const coercions = [];
const numericLeft = {valueOf() { coercions.push('left'); return 2; }};
const numericRight = {valueOf() { coercions.push('right'); return 3; }};
function check(ok) { if (!ok) throw 'retained binding policy'; }
function checkType(error) { check(Object.getPrototypeOf(error) === intrinsicTypePrototype); }
async function plain() {
  const value = 0;
  const present = 1;
  const absent = null;
  const numeric = numericLeft;
  let calls = 0;
  function rhs() { calls++; return whole; }
  function numericRhs() { calls++; coercions.push('rhs'); return numericRight; }
  function skipped() { throw 'skipped const RHS'; }
  check((value &&= await skipped()) === 0);
  check((present ||= await skipped()) === 1);
  check((value ??= await skipped()) === 0);
  let caught = 0;
  try { value ||= await rhs(); } catch (error) { checkType(error); caught++; }
  try { present &&= await rhs(); } catch (error) { checkType(error); caught++; }
  try { absent ??= await rhs(); } catch (error) { checkType(error); caught++; }
  try { value = await rhs(); } catch (error) { checkType(error); caught++; }
  coercions.length = 0;
  try { numeric += await numericRhs(); } catch (error) { checkType(error); caught++; }
  check(calls === 5 && caught === 5 && value === 0 && present === 1 && absent === null);
  check(numeric === numericLeft && coercions.join(',') === 'rhs,left,right');
  let mutable = 0;
  check((mutable ||= await whole) === whole && mutable === whole);
}
async function captured() {
  const value = 0;
  const read = () => value;
  let calls = 0;
  function rhs() { calls++; gc(); return whole; }
  let caught = false;
  try { value ||= await rhs(); } catch (error) { checkType(error); caught = true; }
  check(caught && calls === 1 && read() === 0);
}
async function blocksAndHeads() {
  let caught = 0;
  { const value = 0; try { value = await whole; } catch (error) { checkType(error); caught++; } check(value === 0); }
  for (const value of [0, 0]) {
    try { value ||= await whole; } catch (error) { checkType(error); caught++; }
    check(value === 0);
  }
  check(caught === 3);
}
function* sync() {
  const value = 0;
  const numeric = numericLeft;
  let calls = 0;
  function rhs() { calls++; return 'sync'; }
  function numericRhs() { calls++; coercions.push('rhs'); return 'sync'; }
  check((value &&= yield 'skipped') === 0);
  let caught = 0;
  try { value ||= yield rhs(); } catch (error) { checkType(error); caught++; }
  try { value = yield rhs(); } catch (error) { checkType(error); caught++; }
  coercions.length = 0;
  try { numeric += yield numericRhs(); } catch (error) { checkType(error); caught++; }
  check(calls === 3 && caught === 3 && value === 0);
  check(numeric === numericLeft && coercions.join(',') === 'rhs,left,right');
  return whole;
}
async function* mixed() {
  const value = 0;
  const numeric = numericLeft;
  let calls = 0;
  function rhs() { calls++; return 'mixed'; }
  function numericRhs() { calls++; coercions.push('rhs'); return 'mixed'; }
  check((value &&= await (yield 'skipped')) === 0);
  let caught = 0;
  try { value ||= await (yield rhs()); } catch (error) { checkType(error); caught++; }
  try { value = await (yield rhs()); } catch (error) { checkType(error); caught++; }
  coercions.length = 0;
  try { numeric += await (yield numericRhs()); } catch (error) { checkType(error); caught++; }
  check(calls === 3 && caught === 3 && value === 0);
  check(numeric === numericLeft && coercions.join(',') === 'rhs,left,right');
  return whole;
}
const named = async function self() {
  const original = self;
  let calls = 0;
  function rhs() { calls++; return whole; }
  let caught = false;
  let result;
  try { result = self &&= await rhs(); } catch (error) { checkType(error); caught = true; }
  check(caught === strictMode && calls === 1 && self === original);
  check(strictMode || result === whole);
};
const shadowed = async function self() {
  const self = 0;
  let caught = false;
  try { self ||= await whole; } catch (error) { checkType(error); caught = true; }
  check(caught && self === 0);
};
async function run() {
  await plain(); await captured(); await blocksAndHeads(); await named(); await shadowed();
  let iterator = sync();
  check(iterator.next().value === 'sync');
  check(iterator.next(whole).value === 'sync');
  check(iterator.next(whole).value === 'sync');
  let result = iterator.next(numericRight); check(result.done && result.value === whole);
  iterator = mixed();
  check((await iterator.next()).value === 'mixed');
  check((await iterator.next(whole)).value === 'mixed');
  check((await iterator.next(whole)).value === 'mixed');
  result = await iterator.next(numericRight); check(result.done && result.value === whole);
  print('retained-binding-policies:ok');
}
run().catch(error => print('unexpected:' + error));
