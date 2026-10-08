function equal(actual, expected) { if (actual !== expected) throw new Error(String(actual) + ' !== ' + String(expected)); }
function gate() { let resolve, reject; const promise = new Promise((yes, no) => { resolve = yes; reject = no; }); return {promise, resolve, reject}; }
const pending = gate();
let reads = 0, selected = 0, converted = 0, written = 0;
const data = new Proxy({ab: 1}, {ownKeys(target) { reads++; return Reflect.ownKeys(target); }});
const first = {set slot(value) { equal(value, 'ab'); written++; }};
let selectedTarget = first;
function choose() { selected++; return selectedTarget; }
const rawKey = {[Symbol.toPrimitive]() { converted++; return 'slot'; }};
async function write() {
  outer: inner: for (choose()[await pending.promise] in await Promise.resolve(data)) {
    gc(); await Promise.resolve(); break outer;
  }
  equal(written, 1);
}
const completion = write();
Promise.resolve().then(() => {
  equal(selected, 1); equal(converted, 0);
  selectedTarget = {set slot(value) { throw new Error('target was selected twice'); }};
  gc(); pending.resolve(rawKey);
});
completion.then(async () => {
  equal(reads, 1); equal(selected, 1); equal(converted, 1); equal(written, 1);
  const waiting = gate();
  let readFirst, readLater, readHead;
  async function captured() {
    for (let [first, second, later = (readFirst = () => first, readLater = () => later, await waiting.promise)] in
         (readHead = () => first, await Promise.resolve({ab: 1}))) {
      await Promise.resolve(); return () => [first, later];
    }
  }
  const result = captured();
  await Promise.resolve();
  equal(readFirst(), 'a');
  let early = false; try { readLater(); } catch (error) { early = error instanceof ReferenceError; }
  equal(early, true); gc(); waiting.resolve(23);
  const retained = await result; gc(); equal(retained()[0], 'a'); equal(retained()[1], 23);
  equal(readLater(), 23);
  let head = false; try { readHead(); } catch (error) { head = error instanceof ReferenceError; }
  equal(head, true);
  const original = String.prototype[Symbol.iterator];
  let closed = 0, finalized = 0;
  String.prototype[Symbol.iterator] = function () { return {next() { return {done: false, value: undefined}; }, return() { closed++; return {}; }}; };
  const failure = {};
  try {
    async function defaults() { try { for (let [value = await Promise.reject(failure)] in {ab: 1}) { throw new Error('body'); } } finally { finalized++; } }
    let observed; try { await defaults(); } catch (error) { observed = error; }
    equal(observed, failure); equal(closed, 1); equal(finalized, 1);
  } finally { String.prototype[Symbol.iterator] = original; }
  print('ok');
});
