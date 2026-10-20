var NativeTypeError = TypeError, getterError = {}, callError = {};
async function check(kind) {
  var events = [], nexts = 0, gets = 0, calls = 0, caught = null;
  var iterator = {
    next() { nexts++; return { value: 1, done: false }; },
    get return() {
      gets++; events.push("get");
      if (kind === "getter") throw getterError;
      if (kind === "null") return null;
      if (kind === "noncallable") return 1;
      return function () {
        if (this !== iterator) throw "wrong close receiver";
        calls++; events.push("call");
        if (kind === "call") throw callError;
        if (kind === "primitive") return 1;
        return {};
      };
    }
  };
  var iterable = { [Symbol.iterator]() { return iterator; } };
  try {
    for (const value of iterable) {
      try { await 0; break; }
      finally { events.push("inner"); await 0; events.push("inner-done"); }
      throw "break fell through";
    }
    events.push("tail");
  } catch (error) {
    caught = error; events.push("catch"); await 0; events.push("caught");
  } finally {
    events.push("outer"); await 0; events.push("outer-done");
  }
  var called = kind === "call" || kind === "primitive" || kind === "object";
  var failed = kind !== "null" && kind !== "object";
  var expected = "inner,inner-done,get" + (called ? ",call" : "") + (failed ? ",catch,caught" : ",tail") + ",outer,outer-done";
  if (events.join(",") !== expected || nexts !== 1 || gets !== 1 || calls !== (called ? 1 : 0)) throw events.join(",");
  if (kind === "getter" && caught !== getterError) throw "getter identity";
  if (kind === "call" && caught !== callError) throw "call identity";
  if ((kind === "noncallable" || kind === "primitive") && !(caught instanceof NativeTypeError)) throw "native close TypeError";
  if (!failed && caught !== null) throw "unexpected close error";
}
async function run() {
  await check("null"); await check("object"); await check("getter");
  await check("noncallable"); await check("call"); await check("primitive");
}
run().then(function () { print("ok"); }, function (error) { print("failed: " + String(error)); });
