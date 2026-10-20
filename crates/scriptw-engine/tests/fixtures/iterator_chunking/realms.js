var other = __scriptwCreateRealm().global;
var foreignPrototype = Object.getPrototypeOf(other.Iterator.prototype.chunks.call({next() { return {done: true}; }}, 1));
var foreignNext = foreignPrototype.next, foreignReturn = foreignPrototype.return;
for (var name of ['chunks', 'windows']) {
  var local = Iterator.prototype[name], foreign = other.Iterator.prototype[name];
  var closed = 0;
  var helper = foreign.call({next() { return {value: 7, done: false}; }, return() { ++closed; return {}; }}, 1);
  var result = Object.getPrototypeOf(helper).next.call(helper);
  check(Object.getPrototypeOf(helper) === foreignPrototype && result.value instanceof other.Array && !(result.value instanceof Array));
  check(Object.getPrototypeOf(result) === other.Object.prototype);
  var localHelper = local.call({next() { return {value: 8, done: false}; }, return() { return {}; }}, 1);
  var localResult = foreignNext.call(localHelper);
  check(Object.getPrototypeOf(localResult) === Object.prototype && localResult.value instanceof Array && !(localResult.value instanceof other.Array));
  var fresh = local.call({next() { throw 9; }, return() { return {}; }}, 1);
  var unstartedReturn = foreignReturn.call(fresh);
  check(Object.getPrototypeOf(unstartedReturn) === other.Object.prototype && unstartedReturn.done);
  var caught = false;
  try { foreign.call({return() { ++closed; return {}; }}, 0); }
  catch (error) { caught = error instanceof other.RangeError && !(error instanceof RangeError); }
  check(caught && closed === 1);
  var caught = false;
  try { foreign.call(null, 1); } catch (error) { caught = error instanceof other.TypeError; }
  check(caught);
  var caught = false;
  try { foreignNext.call({}); } catch (error) { caught = error instanceof other.TypeError; }
  check(caught);
  var captured = foreign.call({next() { return {value: 3, done: false}; }}, 1);
  var arrayConstructor = other.Array;
  other.Array = function replacement() {};
  check(foreignNext.call(captured).value instanceof arrayConstructor);
  other.Array = arrayConstructor;
}
