const prototype = Temporal.Instant.prototype;
const descriptor = Object.getOwnPropertyDescriptor(prototype, 'toLocaleString');
if (!descriptor || !descriptor.writable || descriptor.enumerable || !descriptor.configurable) throw 'method descriptor';
const method = descriptor.value;
if (typeof method !== 'function' || method.name !== 'toLocaleString' || method.length !== 0) throw 'method metadata';
if (method === Object.prototype.toLocaleString || method === prototype.toString || method.hasOwnProperty('prototype')) throw 'method identity';
for (const key of ['name', 'length']) {
  const attribute = Object.getOwnPropertyDescriptor(method, key);
  if (attribute.writable || attribute.enumerable || !attribute.configurable) throw 'function descriptor';
}
let caught = false;
try { Reflect.construct(method, []); } catch (error) {
  if (!(error instanceof TypeError)) throw error;
  caught = true;
}
if (!caught) throw 'method is constructable';

let observations = 0;
const locales = {get length() { observations++; throw 'locales observed'; }};
const options = {get localeMatcher() { observations++; throw 'options observed'; }};
const instant = new Temporal.Instant(0n);
const proxy = new Proxy(instant, {
  get() { observations++; throw 'receiver get'; },
  getPrototypeOf() { observations++; throw 'receiver prototype'; }
});
const revoked = Proxy.revocable(instant, {});
revoked.revoke();
for (const receiver of [undefined, null, true, 1, 1n, '', Symbol('receiver'), {}, [], function(){}, prototype, Object.create(prototype), proxy, revoked.proxy]) {
  caught = false;
  try { method.call(receiver, locales, options); } catch (error) {
    if (!(error instanceof TypeError) || Object.getPrototypeOf(error) !== TypeError.prototype) throw 'receiver error';
    caught = true;
  }
  if (!caught || observations !== 0) throw 'branding before locales and options';
}
class SubInstant extends Temporal.Instant {}
if (method.call(new SubInstant(0n), 'en-US', {timeZone:'UTC'}) !== method.call(instant, 'en-US', {timeZone:'UTC'})) throw 'subclass branding';
print('ok');
