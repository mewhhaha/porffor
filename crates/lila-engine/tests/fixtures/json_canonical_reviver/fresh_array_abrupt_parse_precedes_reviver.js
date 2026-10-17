var coercions = 0, callbacks = 0, syntaxErrors = 0;
for (var text of ['[1,]', '[[0],]', '[true false]', '[0,\u2028]']) {
  var input = {[Symbol.toPrimitive]: function (hint) {
    if (hint !== 'string') throw 'wrong hint';
    coercions++;
    return text;
  }};
  try { JSON.parse(input, function (key, value) { callbacks++; return value; }); }
  catch (error) { if (error instanceof SyntaxError) syntaxErrors++; else throw error; }
}
var token = {};
var revoked = Proxy.revocable(function () { throw 'revoked target'; }, {});
revoked.revoke();
var caught;
try {
  JSON.parse({[Symbol.toPrimitive]: function () { throw token; }}, revoked.proxy);
} catch (error) { caught = error; }
if (coercions !== 4 || syntaxErrors !== 4 || callbacks !== 0 || caught !== token) throw 'abrupt ordering';
print('fresh-array-abrupt:ok');
