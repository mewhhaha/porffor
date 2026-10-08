function same(actual, expected, label) {
  if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual + ' != ' + expected);
}
const foreign = __lilaCreateRealm().global;
const ForeignPromise = foreign.Promise;
const ForeignStack = foreign.AsyncDisposableStack;
const foreignDispose = ForeignStack.prototype.disposeAsync;
const localDispose = AsyncDisposableStack.prototype.disposeAsync;
const foreignSuppressedPrototype = foreign.SuppressedError.prototype;
const localSuppressedPrototype = SuppressedError.prototype;
const localTypePrototype = TypeError.prototype;
const bodyMarker = new foreign.TypeError('foreign body');
const lowMarker = new foreign.RangeError('foreign low disposal');
const highMarker = new foreign.TypeError('foreign high disposal');
foreign.Promise = foreign.TypeError = foreign.RangeError = foreign.SuppressedError = function wrongIntrinsic() {
  throw new Error('mutable foreign global must not supply intrinsic');
};
async function originalForeignThrow(events) {
  try {
    switch (0) {default: {
      await using chosen = { [Symbol.asyncDispose]() {
        events.push('dispose');
        return ForeignPromise.resolve().then(() => events.push('resumed'));
      } };
      throw bodyMarker;
    }}
  } catch (error) {
    same(error, bodyMarker, 'foreign throw identity survives fulfilled disposal');
    events.push('caught');
    return;
  }
  throw new Error('missing foreign body rejection');
}
async function rejectedBodyAndDisposers(events) {
  let caught;
  try {
    switch (0) {default: {
      await using low = { [Symbol.asyncDispose]() {
        events.push('low');
        return ForeignPromise.reject(lowMarker);
      } };
      await using high = { [Symbol.asyncDispose]() {
        events.push('high');
        return ForeignPromise.reject(highMarker);
      } };
      await ForeignPromise.reject(bodyMarker);
      throw new Error('rejected body resumed normally');
    }}
  } catch (error) {caught = error;}
  same(Object.getPrototypeOf(caught), localSuppressedPrototype, 'implicit suppression uses executing async Realm');
  same(caught.error, lowMarker, 'outer disposal replaces inner suppression');
  same(Object.getPrototypeOf(caught.suppressed), localSuppressedPrototype, 'inner implicit suppression Realm');
  same(caught.suppressed.error, highMarker, 'inner disposal rejection identity');
  same(caught.suppressed.suppressed, bodyMarker, 'original rejected body identity');
}
async function rejectedBreak(events) {
  try {
    switch (0) {default: {
      await using chosen = { [Symbol.asyncDispose]() {events.push('dispose'); return ForeignPromise.reject(highMarker);} };
      events.push('break');
      break;
    }}
    throw new Error('failed disposal kept normal break');
  } catch (error) {same(error, highMarker, 'disposal failure replaces break without wrapping');}
  events.push('caught');
}
async function rejectedReturn(events) {
  switch (0) {default: {
    await using chosen = { [Symbol.asyncDispose]() {events.push('dispose'); return ForeignPromise.reject(lowMarker);} };
    return 42;
  }}
  throw new Error('missing return');
}
async function finallyThrowAndDisposal(events) {
  let caught;
  try {
    switch (0) {default: {
      await using chosen = { [Symbol.asyncDispose]() {events.push('dispose'); return ForeignPromise.reject(highMarker);} };
      try {events.push('break'); break;}
      finally {await 0; events.push('finally'); throw bodyMarker;}
    }}
  } catch (error) {caught = error;}
  same(Object.getPrototypeOf(caught), localSuppressedPrototype, 'finalizer-selected throw is suppressed in executing Realm');
  same(caught.error, highMarker, 'disposal error replaces selected finally throw');
  same(caught.suppressed, bodyMarker, 'awaited finally replacement identity');
}
async function failedAcquisition(events) {
  try {
    switch (0) {default: {
      await using previous = { [Symbol.asyncDispose]() {events.push('previous'); return ForeignPromise.resolve();} };
      await using invalid = {
        [Symbol.asyncDispose]: 1,
        get [Symbol.dispose]() {throw new Error('invalid async method must not fall back');}
      };
      throw new Error('invalid acquisition reached body');
    }}
  } catch (error) {
    same(Object.getPrototypeOf(error), localTypePrototype, 'invalid acquired method TypeError Realm');
    events.push('invalid-caught');
  }
  try {
    switch (0) {default: {
      await using previous = { [Symbol.asyncDispose]() {events.push('previous-getter'); return ForeignPromise.resolve();} };
      await using invalid = {get [Symbol.asyncDispose]() {throw bodyMarker;}};
      throw new Error('abrupt method getter reached body');
    }}
  } catch (error) {same(error, bodyMarker, 'method-getter foreign abrupt identity'); events.push('getter-caught');}
}
async function borrowedDisposal(stack, dispose, promisePrototype, suppressionPrototype, events) {
  const first = {};
  const second = {};
  stack.defer(() => {events.push('first'); throw first;});
  stack.defer(() => {events.push('second'); return ForeignPromise.reject(second);});
  let identity;
  const bridge = { [Symbol.asyncDispose]() {
    const result = dispose.call(stack);
    same(Object.getPrototypeOf(result), promisePrototype, 'borrowed disposal Promise intrinsic Realm');
    return result.catch(error => {identity = error; throw error;});
  } };
  let caught;
  try {
    switch (0) {default: {
      await using chosen = bridge;
      events.push('body');
      break;
    }}
  } catch (error) {caught = error;}
  same(caught, identity, 'switch preserves borrowed rejection identity');
  same(Object.getPrototypeOf(caught), suppressionPrototype, 'borrowed stack suppression defining Realm');
  same(caught.error, first, 'borrowed outer error');
  same(caught.suppressed, second, 'borrowed inner rejection');
}
async function run() {
  let events = [];
  await originalForeignThrow(events);
  same(events.join('|'), 'dispose|resumed|caught', 'foreign throw waits for disposal');
  events = [];
  await rejectedBodyAndDisposers(events);
  same(events.join('|'), 'high|low', 'all rejecting disposers run in reverse order');
  events = [];
  await rejectedBreak(events);
  same(events.join('|'), 'break|dispose|caught', 'failed disposal replaces pending break');
  events = [];
  try {await rejectedReturn(events); throw new Error('failed disposal fulfilled return');}
  catch (error) {same(error, lowMarker, 'disposal failure replaces return identity');}
  same(events.join('|'), 'dispose', 'failed return disposes once');
  events = [];
  await finallyThrowAndDisposal(events);
  same(events.join('|'), 'break|finally|dispose', 'finally selects throw before disposal');
  events = [];
  await failedAcquisition(events);
  same(events.join('|'), 'previous|invalid-caught|previous-getter|getter-caught', 'failed acquisition disposes earlier resources only');
  events = [];
  await borrowedDisposal(new AsyncDisposableStack(), foreignDispose, ForeignPromise.prototype, foreignSuppressedPrototype, events);
  same(events.join('|'), 'body|second|first', 'foreign method on local stack inside switch');
  events = [];
  await borrowedDisposal(new ForeignStack(), localDispose, Promise.prototype, localSuppressedPrototype, events);
  same(events.join('|'), 'body|second|first', 'local method on foreign stack inside switch');
}
run().then(() => print('async-switch-errors-and-realms:ok'), error => print('FAIL: ' + error));
262;
