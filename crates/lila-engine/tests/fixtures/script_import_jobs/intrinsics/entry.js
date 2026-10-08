const foreign = __lilaCreateRealm();
const ForeignPromise = foreign.global.Promise;
const foreignSpecifier = foreign.evalScript("({toString() { return './target.js'; }})");
const OriginalPromise = Promise;
const originalThen = Promise.prototype.then;
const OriginalTypeError = TypeError;
const OriginalSyntaxError = SyntaxError;
const userDispatcher = function () { throw 'observable user dispatcher property'; };
globalThis['$lila$module$import$0'] = userDispatcher;
Promise.prototype.then = function () { throw 'observable internal then'; };
globalThis.Promise = function () { throw 'observable global Promise'; };
globalThis.TypeError = function () { throw 'observable global TypeError'; };
globalThis.SyntaxError = function () { throw 'observable global SyntaxError'; };
Reflect.ownKeys = function () { throw 'observable Reflect.ownKeys'; };
Object.getOwnPropertyDescriptor = function () { throw 'observable descriptor lookup'; };
const result = import(foreignSpecifier, { with: {} });
if (!(result instanceof OriginalPromise) || result instanceof ForeignPromise)
  throw 'wrong import promise realm';
let completed = 0;
function finish() { completed++; if (completed === 3) print('Script import realm intrinsics'); }
originalThen.call(result, ns => {
  if (ns.value !== 7) throw 'wrong export';
  if (globalThis['$lila$module$import$0'] !== userDispatcher)
    throw 'private dispatcher replaced user property data';
  finish();
});
originalThen.call(import('./target.js', null), () => { throw 'invalid options fulfilled'; }, error => {
  if (!(error instanceof OriginalTypeError)) throw 'wrong options error realm';
  finish();
});
originalThen.call(import('./invalid.js'), () => { throw 'invalid syntax fulfilled'; }, error => {
  if (!(error instanceof OriginalSyntaxError)) throw 'wrong syntax error realm';
  finish();
});
true;
