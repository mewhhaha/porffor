function same(actual, expected, label) {
  if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual + ' != ' + expected);
}
function asynchronous(events, name) {
  return { [Symbol.asyncDispose]() {
    events.push(name + ':dispose');
    return Promise.resolve().then(() => events.push(name + ':resumed'));
  } };
}
async function lifoAndFallthrough(events) {
  let poisonReads = 0;
  let preferredReads = 0;
  let fallbackReads = 0;
  const poison = {get then() {poisonReads++; throw new Error('fallback return must be discarded');}};
  const preferred = {
    get [Symbol.asyncDispose]() {
      preferredReads++;
      events.push('acquire:preferred');
      return function () {
        same(this, preferred, 'cached async disposer receiver');
        events.push('preferred:dispose');
        return Promise.resolve().then(() => events.push('preferred:resumed'));
      };
    },
    get [Symbol.dispose]() {throw new Error('async method must win');}
  };
  const fallback = {
    get [Symbol.asyncDispose]() {events.push('acquire:fallback-async'); return undefined;},
    get [Symbol.dispose]() {
      fallbackReads++;
      events.push('acquire:fallback-sync');
      return function () {
        same(this, fallback, 'cached fallback receiver');
        events.push('fallback:dispose');
        return poison;
      };
    }
  };
  switch (0) {
    case 0: {
      using first = { [Symbol.dispose]() {events.push('first:dispose');} };
      await using second = preferred;
      await using third = fallback;
      Object.defineProperty(preferred, Symbol.asyncDispose, {value() {throw new Error('method was re-read');}, configurable: true});
      Object.defineProperty(fallback, Symbol.dispose, {value() {throw new Error('fallback was re-read');}, configurable: true});
      {
        await using inner = asynchronous(events, 'inner');
        events.push('inner:body');
      }
      events.push('outer:body');
    }
    case 1: {
      using tail = { [Symbol.dispose]() {events.push('tail:dispose');} };
      events.push('tail:body');
      break;
    }
  }
  events.push('after');
  same(preferredReads, 1, 'async method acquired once');
  same(fallbackReads, 1, 'fallback method acquired once');
  same(poisonReads, 0, 'fallback return is not assimilated');
}
async function pendingReturn(events) {
  const returned = {then(resolve) {events.push('adopt'); resolve(42);}};
  switch (0) {default: {
    await using chosen = asynchronous(events, 'resource');
    try {events.push('body'); return returned;}
    finally {await 0; events.push('finally');}
  }}
  throw new Error('pending return lost');
}
async function breakReplacedByReturn(events) {
  switch (0) {default: {
    await using chosen = asynchronous(events, 'resource');
    try {events.push('body'); break;}
    finally {await 0; events.push('finally'); return 'replacement';}
  }}
  throw new Error('replaced break reached switch exit');
}
async function returnReplacedByBreak(events) {
  switch (0) {default: {
    await using chosen = asynchronous(events, 'resource');
    try {events.push('body'); return 'lost';}
    finally {await 0; events.push('finally'); break;}
  }}
  events.push('after');
  return 'after';
}
async function nestedFinalizers(events) {
  chosen: switch (0) {default: {
    await using outer = asynchronous(events, 'outer');
    try {
      {
        await using inner = asynchronous(events, 'inner');
        try {events.push('body'); break chosen;}
        finally {await 0; events.push('inner-finally');}
      }
    } finally {await 0; events.push('outer-finally');}
  }}
  events.push('after');
}
async function run() {
  let events = [];
  await lifoAndFallthrough(events);
  same(events.join('|'), 'acquire:preferred|acquire:fallback-async|acquire:fallback-sync|inner:body|inner:dispose|inner:resumed|outer:body|fallback:dispose|preferred:dispose|preferred:resumed|first:dispose|tail:body|tail:dispose|after', 'nested LIFO and fallthrough');
  events = [];
  same(await pendingReturn(events), 42, 'return payload survives disposal');
  same(events.join('|'), 'body|finally|resource:dispose|resource:resumed|adopt', 'finally and disposal precede return adoption');
  events = [];
  same(await breakReplacedByReturn(events), 'replacement', 'awaited finally selects return');
  same(events.join('|'), 'body|finally|resource:dispose|resource:resumed', 'replacement still disposes once');
  events = [];
  same(await returnReplacedByBreak(events), 'after', 'awaited finally selects local switch break');
  same(events.join('|'), 'body|finally|resource:dispose|resource:resumed|after', 'selected break dispatches after disposal');
  events = [];
  await nestedFinalizers(events);
  same(events.join('|'), 'body|inner-finally|inner:dispose|inner:resumed|outer-finally|outer:dispose|outer:resumed|after', 'labelled switch target stays live through nested finalizers');
}
run().then(() => print('async-switch-completion-and-disposal:ok'), error => print('FAIL: ' + error));
262;
