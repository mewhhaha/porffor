async function choose(first, second) {
  let checks = 0;
  function test(value) { checks++; return value; }
  const result = test(first) ? (test(second) ? await 3 : 4) : await 5;
  await 0;
  return result + ':' + checks;
}
async function run() {
  const first = await choose(true, true);
  const second = await choose(true, false);
  const third = await choose(false, true);
  if (first !== '3:2' || second !== '4:2' || third !== '5:1') throw 'nested branch selection';
  let testCount = 0;
  const tested = (await {get then() { testCount++; return resolve => resolve(false); }})
    ? await (() => { throw 'unchosen arm after awaited test'; })() : await 10;
  if (tested !== 10 || testCount !== 1) throw 'awaited condition once';
  let shared = 0;
  const left = false ? (shared = [1], await 0) : (shared = 'abc', await 1);
  if (left !== 1 || shared.length !== 3) throw 'conditional facts';
  const retained = true ? await {value: 7} : 0;
  const later = retained.value + await 2;
  if (later !== 9) throw 'result retention';
  var outer = await (false ? await 1 : 8);
  if (outer !== 8) throw 'outer await target';
  let observed = 0;
  const initialized = true ? await {get then() {
    try { (() => initialized)(); } catch (error) {
      if (Object.getPrototypeOf(error) !== ReferenceError.prototype) throw 'TDZ Realm';
      observed++;
    }
    return resolve => resolve(9);
  }} : 0;
  if (initialized !== 9 || observed !== 1) throw 'lexical initialization';
  const nested = true ? async () => await 1 : async () => await 2;
  if (await nested() !== 1) throw 'nested function activation';
  const Class = class { [true ? await 'key' : 'other']() { return 4; }
    async separate() { await 0; return 5; } };
  if (new Class().key() !== 4) throw 'computed class name';
}
run().then(() => print('nested:ok'), error => print('unexpected:' + error));
