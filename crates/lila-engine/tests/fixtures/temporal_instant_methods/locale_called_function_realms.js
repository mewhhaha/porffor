const foreign = __lilaCreateRealm().global;
const Instant = foreign.Temporal.Instant;
const method = Instant.prototype.toLocaleString;
const mainMethod = Temporal.Instant.prototype.toLocaleString;
const typeErrorPrototype = foreign.TypeError.prototype;
const rangeErrorPrototype = foreign.RangeError.prototype;
if (!Instant.prototype.hasOwnProperty('toLocaleString') || method === mainMethod || Object.getPrototypeOf(method) !== foreign.Function.prototype) throw 'foreign callable';
const receiver = new Temporal.Instant(0n);
const foreignReceiver = new Instant(0n);
const expected = new Intl.DateTimeFormat('en-US', {timeZone:'UTC'}).format(receiver);
if (method.call(receiver, 'en-US', {timeZone:'UTC'}) !== expected || mainMethod.call(foreignReceiver, 'en-US', {timeZone:'UTC'}) !== expected) throw 'borrowed branded receiver';

let boxedReads = 0;
Object.defineProperty(foreign.Number.prototype, 'timeZone', {configurable:true, get(){
  if (Object.getPrototypeOf(this) !== foreign.Number.prototype) throw 'options wrapper prototype';
  boxedReads++;
  return 'UTC';
}});
Object.defineProperty(Number.prototype, 'timeZone', {configurable:true, get(){throw 'entry options wrapper';}});
if (method.call(receiver, 'en-US', 1) !== expected || boxedReads !== 1) throw 'called function option boxing';
delete Number.prototype.timeZone;

Object.defineProperty(foreign.Intl, 'DateTimeFormat', {configurable:true, get(){throw 'public foreign formatter';}});
foreign.Temporal.Instant = function(){throw 'public foreign Instant';};
foreign.TypeError = function(){throw 'public foreign TypeError';};
foreign.RangeError = function(){throw 'public foreign RangeError';};
if (method.call(receiver, 'en-US', {timeZone:'UTC'}) !== expected) throw 'foreign intrinsic formatter';
for (const operation of [
  () => method.call({}, {get length(){throw 'locales after wrong receiver';}}, {get localeMatcher(){throw 'options after wrong receiver';}}),
  () => method.call(receiver, null, {get localeMatcher(){throw 'options after null locales';}}),
  () => method.call(receiver, 'en-US', null),
  () => method.call(receiver, 'en-US', {timeZone:Symbol('zone')})
]) {
  let caught = false;
  try { operation(); } catch (error) {
    if (Object.getPrototypeOf(error) !== typeErrorPrototype || error instanceof TypeError) throw 'called function TypeError Realm';
    caught = true;
  }
  if (!caught) throw 'missing TypeError';
}
for (const operation of [
  () => method.call(receiver, 'not_a_locale'),
  () => method.call(receiver, 'en-US', {timeZone:'Definitely/Invalid_Zone'}),
  () => method.call(receiver, 'en-US', {fractionalSecondDigits:4})
]) {
  let caught = false;
  try { operation(); } catch (error) {
    if (Object.getPrototypeOf(error) !== rangeErrorPrototype || error instanceof RangeError) throw 'called function RangeError Realm';
    caught = true;
  }
  if (!caught) throw 'missing RangeError';
}
let caught = false;
try { mainMethod.call({}, 'en-US'); } catch (error) {
  if (Object.getPrototypeOf(error) !== TypeError.prototype) throw 'entry error Realm';
  caught = true;
}
if (!caught) throw 'missing entry TypeError';
const marker = {};
caught = false;
try { method.call(receiver, 'en-US', {get timeZone(){throw marker;}}); } catch (error) {
  if (error !== marker) throw 'borrowed method abrupt identity';
  caught = true;
}
if (!caught) throw 'missing abrupt';
print('ok');
