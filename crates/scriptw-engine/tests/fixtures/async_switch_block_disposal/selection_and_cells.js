function same(actual, expected, label) {
  if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual + ' != ' + expected);
}
function record(events, name, value) {events.push(name); return value;}
function resource(events, name) {
  return { [Symbol.asyncDispose]() {
    events.push(name + ':dispose');
    return Promise.resolve().then(() => events.push(name + ':resumed'));
  } };
}
async function selected(events) {
  switch (await record(events, 'discriminant', 2)) {
    case await record(events, 'test0', 0): {
      await using skipped = resource(events, 'wrong0');
      throw new Error('unselected body');
    }
    default: {
      await using skipped = resource(events, 'wrong-default');
      throw new Error('default before later match');
    }
    case await record(events, 'test2', 2): {
      await using chosen = resource(events, 'chosen');
      events.push('chosen:body');
    }
    case record(events, 'wrong-test3', 3): {
      await using tail = resource(events, 'tail');
      events.push('tail:body');
      break;
    }
  }
  events.push('after');
}
async function defaultFallback(events) {
  switch (9) {
    case await record(events, 'test0', 0): {
      await using skipped = resource(events, 'wrong0');
      throw new Error('unselected first body');
    }
    default: {
      await using chosen = resource(events, 'default');
      events.push('default:body');
    }
    case await record(events, 'test2', 2): {
      await using tail = resource(events, 'tail');
      events.push('tail:body');
      break;
    }
  }
  events.push('after');
}
async function noMatch(events) {
  switch (9) {
    case await record(events, 'test0', 0): {
      await using skipped = resource(events, 'wrong0');
      throw new Error('unselected no-match body');
    }
    case await record(events, 'test1', 1): {
      await using skipped = resource(events, 'wrong1');
      throw new Error('unselected no-match body');
    }
  }
  events.push('after');
}
let savedReader;
let skippedReader;
async function caseCells(events) {
  switch (0) {
    case 0:
      let cell = 1;
      const read = () => cell;
      skippedReader = () => later;
      {
        let blockCell = 10;
        await using chosen = { [Symbol.asyncDispose]() {
          events.push('dispose:' + read() + ':' + blockCell);
          return Promise.resolve().then(() => {
            same(read(), 2, 'case cell while disposal is suspended');
            same(blockCell, 11, 'block cell while disposal is suspended');
            events.push('disposed');
          });
        } };
        cell = 2;
        blockCell = 11;
        await 0;
        events.push('body:' + read() + ':' + blockCell);
      }
      cell = 3;
      savedReader = read;
    case 1:
      events.push('tail:' + read());
      break;
    case 2:
      let later = 9;
      throw new Error('unselected lexical initializer');
  }
  same(savedReader(), 3, 'retained case closure after exit');
  try {skippedReader(); throw new Error('missing skipped-case TDZ');}
  catch (error) {same(error instanceof ReferenceError, true, 'skipped case cell remains TDZ');}
}
async function nullOnly(value, events) {
  // This function has no explicit Await expression. The nullish record owns
  // the actual implicit disposal continuation before the following statement.
  switch (0) {default: {
    await using empty = value;
    events.push('body');
  }}
  events.push('after');
}
async function run() {
  let events = [];
  let pending = selected(events);
  events.push('caller');
  await pending;
  same(events.join('|'), 'discriminant|caller|test0|test2|chosen:body|chosen:dispose|chosen:resumed|tail:body|tail:dispose|tail:resumed|after', 'selection and fallthrough disposal');
  events = [];
  pending = defaultFallback(events);
  events.push('caller');
  await pending;
  same(events.join('|'), 'test0|caller|test2|default:body|default:dispose|default:resumed|tail:body|tail:dispose|tail:resumed|after', 'default follows all failed tests');
  events = [];
  await noMatch(events);
  same(events.join('|'), 'test0|test1|after', 'no-match skips acquisition and disposal');
  events = [];
  await caseCells(events);
  same(events.join('|'), 'body:2:11|dispose:2:11|disposed|tail:3', 'shared and block cells survive disposal');
  events = [];
  pending = nullOnly(null, events);
  same(events.join('|'), 'body', 'null disposal suspends before exit');
  events.push('caller');
  await pending;
  same(events.join('|'), 'body|caller|after', 'null implicit Await order');
  events = [];
  pending = nullOnly(undefined, events);
  same(events.join('|'), 'body', 'undefined disposal suspends before exit');
  events.push('caller');
  await pending;
  same(events.join('|'), 'body|caller|after', 'undefined implicit Await order');
}
run().then(() => print('async-switch-selection-and-cells:ok'), error => print('FAIL: ' + error));
262;
