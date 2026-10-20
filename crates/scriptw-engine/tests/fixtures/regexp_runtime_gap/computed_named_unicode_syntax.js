function expression(units, flags) { return new RegExp(fromUnits(units), fromUnits(flags)); }
function capture(name, body) { return [40, 63, 60].concat(name, [62], body, [41]); }
function reference(name) { return [92, 107, 60].concat(name, [62]); }
function rejects(units, flags) {
  var error;
  try { expression(units, flags); } catch (caught) { error = caught; }
  require(error instanceof SyntaxError, 'invalid Unicode named grammar must throw SyntaxError');
}
for (var mode = 117; mode <= 118; mode++) {
  rejects(reference([109, 105, 115, 115, 105, 110, 103]), [mode]);
  rejects([40, 97, 41].concat(reference([109, 105, 115, 115, 105, 110, 103])), [mode]);
  rejects(reference([109, 105, 115, 115, 105, 110, 103]).concat(capture([111, 116, 104, 101, 114], [97])), [mode]);
  for (var malformed of [
    [92, 107],
    [92, 107, 60],
    [92, 107, 60, 97],
    [92, 107, 60, 62],
    [92, 107, 60, 49, 62],
    [92, 107, 60, 97, 33, 62],
    [92, 107, 60, 92, 120, 54, 49, 62],
    [92, 107, 60, 92, 117, 123, 125, 62],
    [92, 107, 60, 92, 117, 123, 49, 49, 48, 48, 48, 48, 125, 62],
    [92, 107, 60, 92, 117, 123, 51, 69, 125, 62],
    [92, 107, 60, 92, 117, 123, 68, 56, 48, 48, 125, 62],
    [92, 107, 60, 92, 117, 68, 56, 48, 49, 62],
    [92, 107, 60, 92, 117, 68, 56, 48, 49, 92, 117, 123, 68, 67, 48, 48, 125, 62],
    [92, 107, 60, 0xdc00, 62]
  ]) rejects(malformed, [mode]);
  for (var invalidClass of [
    [91, 92, 107, 93],
    [91, 92, 107, 60, 97, 62, 93],
    [91, 92, 107, 45, 109, 93],
    [91, 105, 45, 92, 107, 93]
  ]) rejects(invalidClass, [mode]);
  rejects([91, 92, 107, 93].concat(capture([97], [120])), [mode]);
  rejects(capture([115, 97, 109, 101], [97]).concat(capture([115, 97, 109, 101], [98]), reference([115, 97, 109, 101])), [mode]);
  rejects(capture([97], [97]).concat(capture([92, 117, 123, 54, 49, 125], [98]), reference([97])), [mode]);
  rejects(capture([0x212a], [97]).concat(reference([75])), [105, mode]);
  var receiver = /old/g;
  receiver.lastIndex = 7;
  var failed;
  try { receiver.compile(fromUnits(reference([109, 105, 115, 115, 105, 110, 103])), fromUnits([mode])); }
  catch (error) { failed = error instanceof SyntaxError; }
  require(failed && receiver.source === 'old' && receiver.flags === 'g' && receiver.lastIndex === 7, 'unknown Unicode name cannot install a partial program');
  receiver.lastIndex = 0;
  require(receiver.test('old') && receiver.lastIndex === 3, 'previous program still matches after failed named compilation');
  receiver.compile(fromUnits(capture([110], [97]).concat(reference([110]))), fromUnits([100, mode]));
  var recovered = receiver.exec('aa');
  require(receiver.lastIndex === 0 && recovered && recovered.groups.n === 'a' && recovered.indices.groups.n === recovered.indices[1], 'a valid named compile recovers after syntax rollback');
}
rejects([91, 92, 107, 93].concat(capture([97], [120])), []);
require(expression([94, 92, 107, 60, 49, 62, 36], []).test('k<1>'), 'Legacy unknown-looking name remains identity text without declarations');
print('ok');
262;
