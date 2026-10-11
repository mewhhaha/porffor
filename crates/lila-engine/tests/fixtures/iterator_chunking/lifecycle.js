for (var method of [Iterator.prototype.chunks, Iterator.prototype.windows]) {
  var closed = 0, steps = 0;
  var source = {next() { ++steps; return {value: steps, done: false}; }, return() { ++closed; return {}; }};
  var unstarted = method.call(source, 2);
  check(unstarted.return().done && unstarted.next().done && unstarted.return().done && closed === 1 && steps === 0);
  var started = method.call(source, 2);
  check(!started.next().done && steps === 2);
  check(started.return().done && started.next().done && closed === 2);
  var marker = {};
  for (var stage of ['next', 'done', 'value']) {
    var calls = 0, closes = 0;
    var abrupt = {next() {
      ++calls;
      if (stage === 'next') throw marker;
      return {get done() { if (stage === 'done') throw marker; return false; }, get value() { throw marker; }};
    }, return() { ++closes; return {}; }};
    var helper = method.call(abrupt, 2);
    var caught = false;
    try { helper.next(); } catch (error) { caught = error === marker; }
    check(caught && helper.next().done && helper.return().done && calls === 1 && closes === 0);
  }
  var helper;
  var reentries = 0;
  var source = {next() {
    for (var operation of ['next', 'return']) {
      var caught = false;
      try { helper[operation](); } catch (error) { caught = error instanceof TypeError; }
      check(caught);
      ++reentries;
    }
    return {value: 8, done: false};
  }, return() { return {}; }};
  helper = method.call(source, 1);
  check(helper.next().value[0] === 8 && reentries === 2);
  helper.return();
  var closed = 0;
  var source = {next() { return {done: true}; }, return() { ++closed; return {}; }};
  var exhausted = method.call(source, 2);
  check(exhausted.next().done && exhausted.return().done && closed === 0);
  var count = 0;
  var source = {next() { return count++ === 0 ? {value: 4, done: false} : {done: true}; }, return() { ++closed; return {}; }};
  var partial = method.call(source, 2, 'allow-partial');
  check(!partial.next().done && partial.return().done && closed === 0);
}
var count = 0, marker = {}, closed = 0;
var mapped = Iterator.prototype.map.call({next() { ++count; throw marker; }, return() { ++closed; return {}; }}, function (value) { return value; });
var caught = false;
try { mapped.next(); } catch (error) { caught = error === marker; }
check(caught && mapped.next().done && mapped.return().done && count === 1 && closed === 0);
