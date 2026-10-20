var set = Object.getOwnPropertyDescriptor(Error.prototype, 'stack').set;
var conversions = 0;
for (var value of [undefined, null, 0, true, Symbol(), 1n, new String('text'), {toString() { ++conversions; return 'text'; }}]) {
  var threw = false;
  try { set.call({}, value); } catch (error) { threw = error instanceof TypeError; }
  check(threw);
}
check(conversions === 0);
for (var receiver of [undefined, null, 1, false, '', Symbol(), 1n]) {
  var threw = false;
  try { set.call(receiver, 'text'); } catch (error) { threw = error instanceof TypeError; }
  check(threw);
}
var threw = false;
try { set.call(Error.prototype, 'text'); } catch (error) { threw = error instanceof TypeError; }
check(threw);
for (var object of [{}, Object.create(null), TypeError.prototype]) {
  check(set.call(object, '') === undefined);
  var own = Object.getOwnPropertyDescriptor(object, 'stack');
  check(own.value === '' && own.writable && own.enumerable && own.configurable);
}
var object = {};
Object.defineProperty(object, 'stack', {value: 'old', writable: true, enumerable: false, configurable: false});
set.call(object, 'new');
var own = Object.getOwnPropertyDescriptor(object, 'stack');
check(own.value === 'new' && own.writable && !own.enumerable && !own.configurable);
var received;
var accessor = {set stack(value) { received = value; }};
set.call(accessor, 'accessor');
check(received === 'accessor' && typeof Object.getOwnPropertyDescriptor(accessor, 'stack').set === 'function');
for (var receiver of [Object.preventExtensions({}), Object.defineProperty({}, 'stack', {value: 'fixed', writable: false})]) {
  var threw = false;
  try { set.call(receiver, 'text'); } catch (error) { threw = error instanceof TypeError; }
  check(threw);
}
var log = [];
var target = {};
var proxy = new Proxy(target, {
  getOwnPropertyDescriptor(target, key) { log.push('own:' + key); return Object.getOwnPropertyDescriptor(target, key); },
  defineProperty(target, key, descriptor) { log.push('define:' + key); return Reflect.defineProperty(target, key, descriptor); },
  set(target, key, value) { log.push('set:' + key); target[key] = value; return true; }
});
set.call(proxy, 'first');
set.call(proxy, 'second');
check(log.join('|') === 'own:stack|define:stack|own:stack|set:stack' && target.stack === 'second');
var marker = {};
for (var trap of ['getOwnPropertyDescriptor', 'defineProperty']) {
  var handler = {};
  handler[trap] = function () { throw marker; };
  var caught = false;
  try { set.call(new Proxy({}, handler), 'text'); } catch (error) { caught = error === marker; }
  check(caught);
}
var threw = false;
try { set.call(new Proxy({}, {defineProperty() { return false; }}), 'text'); } catch (error) { threw = error instanceof TypeError; }
check(threw);
