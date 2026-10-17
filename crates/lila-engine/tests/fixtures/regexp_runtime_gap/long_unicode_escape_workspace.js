var zeros = '0'.repeat(Math.pow(2, 24));
var escaped = fromUnits([92, 117, 123]) + zeros + '1234}';
var minimal = fromUnits([92, 117, 123]) + '1234}';
var target = fromUnits([0x1234]);
for (var flags of ['u', 'v']) {
  for (var pair of [[escaped, minimal], ['[' + escaped + ']', '[' + minimal + ']']]) {
    var actual = new RegExp(pair[0], flags);
    var reference = new RegExp(pair[1], flags);
    require(actual.source === pair[0], 'long original source code units are retained');
    require(reference.source === pair[1], 'minimal original source code units are retained');
    require(actual.test(target) === reference.test(target) && actual.test(target), 'one long escape has the same character meaning');
    require(actual.test('x') === reference.test('x') && !actual.test('x'), 'long escape excludes nonmembers');
    var result = actual.exec('a' + target + 'b');
    require(result && result[0] === target && result.index === 1, 'UTF-16 matching indices survive scratch compaction');
  }
}
print('ok');
262;
