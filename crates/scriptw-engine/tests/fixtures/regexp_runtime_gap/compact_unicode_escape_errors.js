var zeros = '0'.repeat(4097);
var prefix = fromUnits([92, 117, 123]);
for (var flags of ['u', 'v']) {
  for (var malformed of [prefix + '}', prefix + zeros, prefix + zeros + 'G}', prefix + zeros + '110000}', prefix + zeros + '-1}', prefix + zeros + ' 41}']) {
    for (var source of [malformed, '[' + malformed + ']']) {
      var rejected = false;
      try { new RegExp(source, flags); } catch (error) { rejected = error instanceof SyntaxError; }
      require(rejected, 'malformed and out of range escapes remain SyntaxError');
    }
  }
}
var qSource = '[' + fromUnits([92]) + 'q{' + prefix + zeros + '110000}}]';
var qRejected = false;
try { new RegExp(qSource, 'v'); } catch (error) { qRejected = error instanceof SyntaxError; }
require(qRejected, 'out of range class-string escape remains SyntaxError');
print('ok');
262;
