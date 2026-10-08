const realm = __lilaCreateRealm();
const other = realm.global;
const foreignArrayPrototype = other.Array.prototype;
const entries = realm.evalScript("function ordinary(value) { return [this, new.target, value]; } function* generator() { yield [this, new.target]; return eval('() => [this, new.target]'); } async function asynchronous(value) { await 0; return [this, new.target, value]; } async function* asyncGenerator() { yield [this, new.target]; await 0; return eval('() => [this, new.target]'); } ({ordinary, generator, asynchronous, asyncGenerator});");
other.Array = function wrongArray() {throw 'mutable Array constructor';};
other.Function = function wrongFunction() {throw 'mutable Function constructor';};
const receiver = {marker: 17};
function Alternate() {}
const called = entries.ordinary.call(receiver, 19);
const constructed = Reflect.construct(entries.ordinary, [23], Alternate);
if (called[0] !== receiver || called[1] !== undefined || called[2] !== 19 || Object.getPrototypeOf(called) !== foreignArrayPrototype) throw 'prepared ordinary call entry';
if (Object.getPrototypeOf(constructed[0]) !== Alternate.prototype || constructed[1] !== Alternate || constructed[2] !== 23 || Object.getPrototypeOf(constructed) !== foreignArrayPrototype) throw 'prepared ordinary construct entry';
const sync = entries.generator.call(receiver);
const syncFirst = sync.next();
const syncLast = sync.next();
const syncCapture = syncLast.value.call({marker: 99});
if (syncFirst.done || !syncLast.done || syncFirst.value[0] !== receiver || syncFirst.value[1] !== undefined || syncCapture[0] !== receiver || syncCapture[1] !== undefined) throw 'prepared generator entry and escaping eval';
if (Object.getPrototypeOf(syncFirst.value) !== foreignArrayPrototype || Object.getPrototypeOf(syncCapture) !== foreignArrayPrototype) throw 'prepared generator defining Realm';
async function observe() {
  const asyncResult = await entries.asynchronous.call(receiver, 29);
  if (asyncResult[0] !== receiver || asyncResult[1] !== undefined || asyncResult[2] !== 29 || Object.getPrototypeOf(asyncResult) !== foreignArrayPrototype) throw 'prepared async body entry';
  const iterator = entries.asyncGenerator.call(receiver);
  const first = await iterator.next();
  const last = await iterator.next();
  const capture = last.value.call({marker: 99});
  if (first.done || !last.done || first.value[0] !== receiver || first.value[1] !== undefined || capture[0] !== receiver || capture[1] !== undefined) throw 'prepared async generator activation role';
  if (Object.getPrototypeOf(first.value) !== foreignArrayPrototype || Object.getPrototypeOf(capture) !== foreignArrayPrototype) throw 'prepared async generator defining Realm';
  print('prepared-callable-entry-roles:ok');
}
observe().catch(error => print('unexpected:' + error));
262;
