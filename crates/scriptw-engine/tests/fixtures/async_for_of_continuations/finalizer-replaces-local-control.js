var returned = {}, thrown = {}, rejected = {}, closeError = {};
function source(closeThrows) {
  var state = { nexts: 0, closes: 0, events: [] };
  state.iterable = { [Symbol.iterator]() { return {
    next() { state.nexts++; return state.nexts <= 2 ? { value: state.nexts, done: false } : { done: true }; },
    return() { state.closes++; state.events.push("close"); if (closeThrows) throw closeError; return {}; }
  }; } };
  return state;
}
async function consume(state, original, replacement) {
  for (const value of state.iterable) {
    try {
      await 0;
      if (original === "continue") continue;
      break;
    } finally {
      state.events.push("finally" + value);
      await 0;
      state.events.push("middle" + value);
      if (replacement === "reject") await Promise.reject(rejected);
      else await 0;
      state.events.push("finalized" + value);
      if (replacement === "continue") continue;
      if (replacement === "break") break;
      if (replacement === "return") return returned;
      if (replacement === "throw") throw thrown;
    }
    throw "branch fell through";
  }
  state.events.push("tail");
  return "tail";
}
async function check(original, replacement, closeThrows, expected, expectedNexts, expectedCloses, trace) {
  var state = source(closeThrows), actual;
  try { actual = await consume(state, original, replacement); }
  catch (error) { actual = error; }
  if (actual !== expected || state.nexts !== expectedNexts || state.closes !== expectedCloses || state.events.join(",") !== trace) throw "wrong selected completion: " + state.events.join(",");
}
async function run() {
  await check("break", "continue", true, "tail", 3, 0, "finally1,middle1,finalized1,finally2,middle2,finalized2,tail");
  await check("continue", "break", false, "tail", 1, 1, "finally1,middle1,finalized1,close,tail");
  await check("continue", "return", false, returned, 1, 1, "finally1,middle1,finalized1,close");
  await check("continue", "return", true, closeError, 1, 1, "finally1,middle1,finalized1,close");
  await check("break", "throw", true, thrown, 1, 1, "finally1,middle1,finalized1,close");
  await check("continue", "reject", true, rejected, 1, 1, "finally1,middle1,close");
  await check("break", "reject", true, rejected, 1, 1, "finally1,middle1,close");
}
run().then(function () { print("ok"); }, function (error) { print("failed: " + String(error)); });
