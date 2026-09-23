var count = 37;
var strictResult = eval("'use strict'; for (var count = 0; count < 3; count++) {} count;");
if (strictResult !== 3 || count !== 37) throw new Error('strict direct eval ownership');
var indirectResult = (0, eval)("for (var count = 0; count < 3; count++) {} count;");
if (indirectResult !== 3 || count !== 3) throw new Error('indirect eval ownership');
var realm = __lilaCreateRealm();
realm.global.count = 91;
if (realm.evalScript("for (var count = 0; count < 3; count++) {} count;") !== 3
    || realm.global.count !== 3) throw new Error('Realm Script ownership');
function ordinary() {
  var count = 37;
  for (var count = 0; count < 3; count++) {}
  return count;
}
if (ordinary() !== 3) throw new Error('ordinary function ownership');
var calls = 0;
function custom(eval) {
  var count = 37;
  var result = eval("for (var count = 0;;) { throw 'must not execute'; }");
  return result === 'custom' && count === 37;
}
if (!custom(function (source) { calls++; return 'custom'; }) || calls !== 1)
  throw new Error('non-intrinsic eval callee');
print('ok');
true;
