(function () {
  function check(actual, expected, label) {
    if (actual !== expected) throw new Error(label);
  }
  var samples = ['', 'x', '\u0000', '\ud800', '\udfff', 'a\ud83d\udca9z', '\udfff\ud800'];
  var positions = [-Infinity, -20, -2, -1, -0, 0, 0.9, 1, 2, 20, Infinity, NaN];
  for (var s = 0; s < samples.length; s++) {
    var input = samples[s];
    for (var p = 0; p < positions.length; p++) {
      var position = positions[p];
      var integer = position !== position ? 0 : Math.trunc(position);
      var char = input.charAt(position);
      var atIndex = integer < 0 ? input.length + integer : integer;
      check(char, integer >= 0 && integer < input.length ? input[integer] : '', 'charAt unit');
      check(input.at(position), atIndex >= 0 && atIndex < input.length ? input[atIndex] : undefined, 'at unit');
      if (char.length) check(char.length, 1, 'one UTF16 unit');
    }
    var iterator = input[Symbol.iterator]();
    var joined = '';
    var previous;
    for (var result = iterator.next(); !result.done; result = iterator.next()) {
      check(result === previous, false, 'fresh iterator result');
      previous = result;
      joined += result.value;
    }
    check(joined, input, 'iterator exact units');
    var done = iterator.next();
    check(done.done, true, 'iterator remains exhausted');
    check(done.value, undefined, 'exhausted value');
    check(done === result, false, 'fresh exhausted result');
  }
  var order = '';
  var receiver = {toString: function () { order += 'r'; return 'a\ud83d\udca9z'; }};
  var index = {valueOf: function () { order += 'i'; return 1; }};
  check(String.prototype.charAt.call(receiver, index), '\ud83d', 'coerced high surrogate');
  check(order, 'ri', 'charAt coercion order');
  order = '';
  index.valueOf = function () { order += 'i'; return -2; };
  check(String.prototype.at.call(receiver, index), '\udca9', 'coerced low surrogate');
  check(order, 'ri', 'at coercion order');
  var sentinel = {};
  index.valueOf = function () { throw sentinel; };
  var caught = false;
  try { String.prototype.at.call(receiver, index); } catch (e) { caught = e === sentinel; }
  check(caught, true, 'index throw identity');
  for (var method of [String.prototype.charAt, String.prototype.at]) {
    caught = false;
    try { method.call(null, index); } catch (e) { caught = e instanceof TypeError; }
    check(caught, true, 'null receiver checked before index');
    caught = false;
    try { method.call('abc', 1n); } catch (e) { caught = e instanceof TypeError; }
    check(caught, true, 'BigInt index rejected');
  }
  var next = Object.getPrototypeOf(''[Symbol.iterator]()).next;
  caught = false;
  try { next.call({}); } catch (e) { caught = e instanceof TypeError; }
  check(caught, true, 'iterator receiver brand');
  var boxed = new String('abc');
  var invalid = ['', '-0', '00', '01', '1.0', '1e0', '+1', '0x1', '1 ', ' 1', '4294967294', '4294967295', '9999999999', '10000000000', '\uff11'];
  for (var k = 0; k < invalid.length; k++) {
    check(Object.getOwnPropertyDescriptor(boxed, invalid[k]), undefined, 'nonindex string descriptor');
  }
  for (var k = 0; k < 3; k++) {
    var descriptor = Object.getOwnPropertyDescriptor(boxed, String(k));
    check(descriptor.value, 'abc'[k], 'indexed string descriptor');
    check(descriptor.writable, false, 'string index readonly');
    check(descriptor.enumerable, true, 'string index enumerable');
    check(descriptor.configurable, false, 'string index nonconfigurable');
  }
  print('single-character-string-reads:ok');
})();
