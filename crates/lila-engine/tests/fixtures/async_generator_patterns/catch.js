function check(condition, label) { if (!condition) throw label; }
var whole = { kind: 'whole' }; whole.self = whole;
async function run() {
  var reads = 0, input = { get selected() { reads++; return undefined; } };
  async function* object() {
    let outside = 'outside';
    try { throw input; }
    catch ({ [await (yield 'key')]: received = await (yield function read() { return [outside, received]; }) }) {
      let outside = 'body'; yield function read() { return [outside, received]; };
    }
    yield outside;
  }
  var iterator = object(); check((await iterator.next()).value === 'key' && reads === 0, 'catch-key-before-original-get');gc();
  var parameterRead = (await iterator.next('selected')).value;
  check(reads === 1, 'catch-get-once-before-default');
  try { parameterRead(); throw 'initialized-catch-parameter'; } catch (error) { check(error instanceof ReferenceError, 'original-catch-parameter-tdz-during-default'); }
  gc(); var bodyRead = (await iterator.next(whole)).value;
  check(parameterRead()[0] === 'outside' && parameterRead()[1] === whole, 'parameter-default-closes-over-original-outer-and-parameter');
  check(bodyRead()[0] === 'body' && bodyRead()[1] === whole && reads === 1, 'body-record-starts-after-binding-initialization');
  check((await iterator.next()).value === 'outside' && (await iterator.next()).done, 'catch-leaves-original-record-chain');gc();
  check(parameterRead()[1] === whole && bodyRead()[1] === whole, 'escaping-parameter-and-body-records-after-completion');

  var events = [], closeError = { close: true }, firstRead;
  function iterable() { return { [Symbol.iterator]: function () { events.push('iterator'); return { next: function () { events.push('next'); return { done: false, value: undefined }; }, return: function () { events.push('close'); if (closeError) throw closeError; return {}; } }; } }; }
  async function* array(input) {
    try { throw input; }
    catch ([received = await (yield function read() { return received; })]) { yield received; }
    finally { await Promise.resolve(0); events.push('finally'); gc(); yield 'finally'; }
  }
  closeError = undefined; iterator = array(iterable()); firstRead = (await iterator.next()).value;
  try { firstRead(); throw 'initialized-array-catch'; } catch (error) { check(error instanceof ReferenceError, 'array-catch-original-tdz'); }
  check((await iterator.next(29)).value === 29 && events.join(',') === 'iterator,next,close', 'array-catch-closes-before-body');
  check((await iterator.next()).value === 'finally' && (await iterator.next()).done && firstRead() === 29, 'array-catch-record-survives-finally');
  events = []; closeError = { close: true }; iterator = array(iterable()); await iterator.next();
  check((await iterator.throw(whole)).value === 'finally' && events.join(',') === 'iterator,next,close,finally', 'binding-injected-throw-closes-before-finalizer');
  try { await iterator.next(); throw 'missing-original-binding-throw'; } catch (error) { check(error === whole, 'original-injected-throw-wins-close-error'); }
  events = []; closeError = undefined; iterator = array(iterable()); await iterator.next();
  check((await iterator.return(whole)).value === 'finally', 'binding-return-runs-original-finalizer');
  var result = await iterator.next(); check(result.done && result.value === whole && events.join(',') === 'iterator,next,close,finally', 'whole-return-survives-binding-close-and-yielding-finally');
  events = []; iterator = array(iterable()); await iterator.next();
  check((await iterator.next(Promise.reject(whole))).value === 'finally', 'binding-await-rejection-enters-finalizer');
  try { await iterator.next(); throw 'missing-default-rejection'; } catch (error) { check(error === whole && events.join(',') === 'iterator,next,close,finally', 'rejected-default-closes-with-whole-original-value'); }
}
run().then(function () { print('mixed-async-generator-catch-patterns:ok'); }, function (error) { print(error); throw error; });
