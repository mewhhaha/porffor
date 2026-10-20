function source(owner) {
  var state = { owner: owner, nexts: 0, closes: 0, events: [], cells: [] };
  var iterator = {
    next() {
      state.nexts++;
      state.events.push("next" + state.nexts);
      return state.nexts <= 4 ? { value: state.nexts, done: false } : { done: true };
    },
    return() { state.closes++; state.events.push("close"); return {}; }
  };
  state.iterator = iterator;
  state.iterable = { [Symbol.iterator]() { return iterator; } };
  return state;
}
async function consume(state) {
  for (let value of state.iterable) {
    let body = state.owner + value;
    state.cells.push(function () { return value + ":" + body; });
    state.events.push("body" + value);
    try {
      if (value === 1) {
        state.iterator.next = function () { throw "next was not cached"; };
        continue;
      }
      await Promise.resolve(value);
      state.events.push("awaited" + value);
      if (value === 2) continue;
      break;
    } finally {
      state.events.push("finally" + value);
      await 0;
      body += "a";
      await 0;
      body += "b";
      state.events.push("finalized" + value);
    }
    throw "local branch fell through";
  }
  state.events.push("tail");
  return state.owner;
}
function check(state) {
  if (state.nexts !== 3 || state.closes !== 1) throw "iteration or close repeated";
  if (state.events.join(",") !== "next1,body1,finally1,finalized1,next2,body2,awaited2,finally2,finalized2,next3,body3,awaited3,finally3,finalized3,close,tail") throw state.events.join(",");
  if (state.cells[0]() !== "1:" + state.owner + "1ab" || state.cells[1]() !== "2:" + state.owner + "2ab" || state.cells[2]() !== "3:" + state.owner + "3ab") throw "iteration cells aliased";
}
async function run() {
  var left = source("L"), right = source("R");
  var results = await Promise.all([consume(left), consume(right)]);
  if (results[0] !== "L" || results[1] !== "R") throw "activation results aliased";
  check(left); check(right);
}
run().then(function () { print("ok"); }, function (error) { print("failed: " + String(error)); });
