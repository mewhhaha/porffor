function check(value, message) { if (!value) throw new Error(message); }
function same(actual, expected, message) { check(actual === expected, message); }
function tdz(read, message) {
  let caught;
  try { read(); } catch (error) { caught = error; }
  check(caught instanceof ReferenceError, message);
}
function never() { throw new Error('skipped operand was evaluated'); }

async function selection() {
  const log = [];
  let outer = 2;
  const receiver = {
    value: 2,
    pick(value) { same(this, receiver, 'optional call receiver'); log.push('call'); return value; }
  };
  const target = { key: 0 };
  let captured;
  async function key() { log.push('key'); return 'key'; }
  async function argument() { log.push('argument'); gc(); return 2; }
  function beforeBody(read) { tdz(read, 'CaseBlock TDZ before selectors'); log.push('first'); return 0; }
  switch (true ? await Promise.resolve(outer) : await never()) {
    case (beforeBody(read), target[await key()] &&= await never()):
      never();
      function read() { return local; }
      break;
    default:
      log.push('default');
    case receiver?.pick(await argument()):
      let local = await Promise.resolve(8);
      captured = read;
      same(read(), 8, 'hoisted function sees actual initialized cell');
      log.push('body');
      break;
    case await never():
      never();
  }
  same(log.join(','), 'first,key,argument,call,body', 'all reached selectors before selected body');
  gc();
  same(captured(), 8, 'CaseBlock capture after cleanup');
  let shortCalls = 0;
  switch (null?.pick(await never()) ?? await Promise.resolve(0)) {
    case null?.[await never()]: never(); break;
    default: shortCalls++;
    case false ? await never() : await Promise.resolve(1): shortCalls++;
  }
  same(shortCalls, 2, 'default midway follows failed lazy selectors then falls through');
  let outerSeen;
  switch (await Promise.resolve(outer)) {
    case 2: let outer = await Promise.resolve(9); outerSeen = outer; break;
  }
  same(outerSeen, 9, 'discriminant evaluates outside CaseBlock');
  same(outer, 2, 'CaseBlock shadow did not replace enclosing cell');
}

async function references() {
  const log = [];
  const original = { value: 1 };
  const replacement = { value: 7 };
  let target = original;
  async function key() { log.push('key'); return 'value'; }
  async function rhs() { target = replacement; log.push('rhs'); gc(); return 2; }
  switch (await Promise.resolve(2)) {
    case target[await key()] &&= await rhs(): log.push('matched'); break;
    default: never();
  }
  same(original.value, 2, 'selected property Reference before suspended RHS');
  same(replacement.value, 7, 'no target reselection after RHS');
  same(log.join(','), 'key,rhs,matched', 'one target key and RHS evaluation');
  const box = { value: null };
  switch (box[await Promise.resolve('value')] ??= await Promise.resolve(4)) {
    case true ? await Promise.resolve(4) : await never(): break;
    default: never();
  }
  same(box.value, 4, 'branch-sensitive discriminant assignment');
}

async function completions() {
  const log = [];
  outer: for (let n = 0; n < 3; n++) {
    try {
      switch (n === 0 ? await Promise.resolve(0) : await Promise.resolve(1)) {
        case true ? await Promise.resolve(0) : await never(): log.push('continue'); continue outer;
        default: never();
        case await Promise.resolve(1): log.push('break'); break outer;
      }
    } finally { await Promise.resolve(); gc(); log.push('finally' + n); }
  }
  same(log.join(','), 'continue,finally0,break,finally1', 'labelled abrupt completion through awaited finalizer');
  const marker = {};
  let escaped;
  let caught;
  try {
    switch (await Promise.resolve(0)) {
      case (escaped = () => local, await Promise.reject(marker)):
        let local = 1;
        never();
    }
  } catch (error) { caught = error; }
  finally { await Promise.resolve(); log.push('rejected-finally'); }
  same(caught, marker, 'rejected selector retains whole thrown value');
  gc();
  tdz(escaped, 'abrupt selector keeps escaped uninitialized CaseBlock cell');
  async function returning() {
    try {
      switch (await Promise.resolve(1)) {
        case await Promise.resolve(1): return marker;
        default: never();
      }
    } finally { await Promise.resolve(); log.push('return-finally'); }
  }
  same(await returning(), marker, 'return identity survives Switch and awaited finally');
  same(log.slice(-2).join(','), 'rejected-finally,return-finally', 'both outward completions clean up once');
}

async function statementLists() {
  const marker = {};
  const log = [];
  let captured;
  async function fallingOff() {
    switch (await Promise.resolve(0)) {
      case await Promise.resolve(0):
        41;
        await Promise.resolve(marker);
        const local = await Promise.resolve(7);
        captured = () => local;
      default:
        try { 99; break; }
        finally { const empty = await Promise.resolve(8); gc(); log.push(empty); }
      case await never(): never();
    }
  }
  same(await fallingOff(), undefined, 'async falloff does not publish internal StatementList value');
  same(captured(), 7, 'selected Empty declaration and fallthrough keep original cells');
  same(log.join(','), '8', 'awaited Empty finalizer runs once');
}

async function main() { await selection(); await references(); await completions(); await statementLists(); print('async-switch-operands:ok'); }
main().catch(error => print('async-switch-operands:error:' + error));
