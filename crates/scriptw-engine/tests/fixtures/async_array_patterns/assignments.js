function check(condition, label) { if (!condition) throw label; }
var whole = { marker: 19 }; whole.self = whole;
var selected = Symbol('selected');
function stream(values, events) {
  var source = { closes: 0 };
  source[Symbol.iterator] = function () {
    events.push('acquire'); var position = 0;
    return { next: function () {
      var index = position++; events.push('step' + index);
      return { done: index >= values.length, value: values[index] };
    }, return: function () { source.closes++; events.push('close'); return {}; } };
  };
  return source;
}
async function run() {
  var events = [], destination = {};
  var rawKey = { [Symbol.toPrimitive]: function () { events.push('key'); return selected; } };
  var input = stream([undefined, whole], events);
  var original = input;
  var returned = ([(await Promise.resolve(destination))[await Promise.resolve(rawKey)] = await Promise.resolve(whole).then(function (value) {
    events.push('default'); gc(); return value;
  }), ...destination.rest] = input);
  check(returned === original && destination[selected] === whole, 'original-whole-rhs-and-selected-target');
  check(destination.rest.length === 1 && destination.rest[0] === whole, 'rest-real-array-whole-values');
  check(events.join(',') === 'acquire,step0,default,key,step1,step2', 'reference-key-conversion-after-default');
  check(input.closes === 0, 'exhausted-rest-no-close');

  var target = destination, replacement = {};
  input = stream([undefined], []);
  var key = { [Symbol.toPrimitive]: function () { return 'kept'; } };
  [target[key] = await Promise.resolve(whole).then(function (value) { target = replacement; gc(); return value; })] = input;
  check(destination.kept === whole && replacement.kept === undefined && input.closes === 1, 'member-reference-not-reselected-after-await');

  var identifier;
  var first = stream([undefined], []), second = stream([undefined], []);
  async function consume(source, value) {
    var local;
    [local = await Promise.resolve(value).then(function (received) { gc(); return received; })] = source;
    return local;
  }
  var one = consume(first, whole), two = consume(second, selected);
  check(await one === whole && await two === selected && first.closes === 1 && second.closes === 1, 'interleaved-activation-records');

  input = stream([undefined], []);
  Object.defineProperty(destination, 'fail', { set: function () { throw whole; } });
  try { [destination.fail = await 1] = input; throw 'missing-put-error'; }
  catch (error) { check(error === whole && input.closes === 1, 'put-error-closes-original-record'); }
  input = stream([undefined], []);
  var nullish = null;
  var converted = 0, badKey = { [Symbol.toPrimitive]: function () { converted++; throw selected; } };
  try { [nullish[badKey] = await 1] = input; throw 'missing-nullish-target'; }
  catch (error) { check(error instanceof TypeError && converted === 0 && input.closes === 1, 'nullish-put-precedes-key-conversion'); }

  input = stream([undefined], []);
  try { [identifier = await Promise.reject(whole)] = input; }
  catch (error) { check(error === whole && input.closes === 1, 'abandoned-reference-retired-on-own-throw'); }
  input = stream([undefined], []);
  [identifier = await Promise.resolve(selected)] = input;
  check(identifier === selected && input.closes === 1, 'fresh-reference-after-caught-rejection');

  class Box {
    #value;
    async take(input) {
      [this.#value = await Promise.resolve(whole)] = input;
      return this.#value;
    }
  }
  var box = new Box(), privateSource = stream([undefined], []);
  check(await box.take(privateSource) === whole && privateSource.closes === 1, 'private-brand-and-original-base');
}
run().then(function () { print('async-array-assignments:ok'); }, function (error) { print(error); throw error; });
