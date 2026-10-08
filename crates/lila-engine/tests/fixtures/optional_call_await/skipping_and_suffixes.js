let skipped = 0;
let iterations = 0;
let getters = 0;
function forbidden() { skipped++; throw 'skipped optional suffix'; }
const forbiddenIterable = {[Symbol.iterator]() { iterations++; throw 'skipped iterator'; }};
const absent = null;
const nullMethod = {get method() { getters++; return null; }};
async function run() {
  const first = absent?.[await forbidden()](...forbiddenIterable, await forbidden()).child[await forbidden()];
  const second = nullMethod.method?.(...forbiddenIterable, await forbidden()).child[await forbidden()];
  const nested = (absent?.[await forbidden()])?.(await forbidden());
  if (first !== undefined || second !== undefined || nested !== undefined || skipped !== 0 || iterations !== 0 || getters !== 1) throw 'whole suffix shorting';

  const trace = [];
  const noncallable = {get method() { trace.push('get'); return 0; }};
  function argument() { trace.push('arg'); return 1; }
  let caught;
  try { noncallable?.method(await argument()); }
  catch (error) { caught = error; trace.push('throw'); }
  if (Object.getPrototypeOf(caught) !== TypeError.prototype || trace.join(',') !== 'get,arg,throw') throw 'ordinary Call must evaluate arguments before rejecting';

  const child = {};
  const replacement = {method() { throw 'replacement child'; }};
  let chosen = child;
  let creates = 0;
  let childGets = 0;
  const factory = {create(value) {
    'use strict';
    if (this !== factory || value !== 5) throw 'factory Reference';
    creates++;
    return chosen;
  }};
  Object.defineProperty(child, 'method', {get() {
    childGets++;
    return function(value) {
      'use strict';
      if (this !== child || value !== 8) throw 'returned-child Reference';
      return 58;
    };
  }});
  const laterKey = {get then() {
    chosen = replacement;
    return resolve => resolve('method');
  }};
  if (factory?.create(await 5)?.[await laterKey](await 8) !== 58 || creates !== 1 || childGets !== 1 || chosen !== replacement) throw 'call result must survive later key suspension';

  let outerArguments = 0;
  function outerArgument() { outerArguments++; return 9; }
  caught = undefined;
  try { (absent?.[await forbidden()]).method(await outerArgument()); }
  catch (error) { caught = error; }
  if (Object.getPrototypeOf(caught) !== TypeError.prototype || outerArguments !== 0) throw 'grouping ends shorting at ordinary property Get';
  caught = undefined;
  try { (absent?.method)(await outerArgument()); }
  catch (error) { caught = error; }
  if (Object.getPrototypeOf(caught) !== TypeError.prototype || outerArguments !== 1 || skipped !== 0) throw 'grouped ordinary call retains argument evaluation';

  const nullChild = {create() { return null; }};
  if (nullChild?.create(await 1)?.[await forbidden()]?.(await forbidden()) !== undefined) throw 'later optional link shorting';
  if (skipped !== 0 || iterations !== 0) throw 'later shorted effects';
}
run().then(() => print('optional-call-suffixes:ok'), error => print('unexpected:' + error));
262;
