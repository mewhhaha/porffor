var scriptVar = 17;
function scriptFunction() { return this; }
let scriptLexical = 23;
const scriptArrow = () => this;
if (this !== globalThis || scriptArrow() !== globalThis) throw 'Script global this';
if (!Object.prototype.hasOwnProperty.call(globalThis, 'scriptVar') ||
    globalThis.scriptVar !== 17 || globalThis.scriptFunction !== scriptFunction)
  throw 'Script declaration ownership';
if (Object.prototype.hasOwnProperty.call(globalThis, 'scriptLexical')) throw 'lexical leaked';
const strictRoot = (function () { return this; })() === undefined;
if (scriptFunction() !== (strictRoot ? undefined : globalThis)) throw 'Script function strictness';
function assertPrivateOwnersHidden() {
  for (const key of Reflect.ownKeys(globalThis)) {
    if (typeof key === 'string' && key.startsWith('$lila$module$'))
      throw 'private module owner leaked through global reflection';
  }
}
assertPrivateOwnersHidden();
if (eval('typeof $lila$module$import$0') !== 'undefined') throw 'private dispatcher leaked into eval';
const pending = import('./target.js');
print('Script globals ready');
pending.then(ns => {
  if (ns.rootThis !== undefined || ns.directArrow() !== undefined ||
      ns.nestedArrow()() !== undefined) throw 'module lexical this';
  const ordinary = ns.ordinary;
  if (ordinary() !== undefined) throw 'module function strictness';
  const receiver = {};
  if (ns.arrowFromCall.call(receiver)() !== receiver) throw 'ordinary activation this';
  if (typeof targetPrivate !== 'undefined' ||
      Object.prototype.hasOwnProperty.call(globalThis, 'targetPrivate')) throw 'module var leaked';
  if (scriptVar !== 17 || scriptLexical !== 23 || this !== globalThis) throw 'Script cells changed';
  assertPrivateOwnersHidden();
  if (eval('typeof $lila$module$import$0') !== 'undefined') throw 'private dispatcher leaked into eval';
  print('module lexical this ready');
});
73;
