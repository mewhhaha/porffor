var descriptor = Object.getOwnPropertyDescriptor(Error.prototype, 'stack');
var get = descriptor.get;
var set = descriptor.set;
check(!descriptor.enumerable && descriptor.configurable && !('value' in descriptor));
check(get.name === 'get stack' && get.length === 0 && set.name === 'set stack' && set.length === 1);
for (var callable of [get, set]) {
  var threw = false;
  try { new callable(); } catch (error) { threw = error instanceof TypeError; }
  check(threw);
}
for (var primitive of [undefined, null, 0, true, '', Symbol(), 1n]) {
  var threw = false;
  try { get.call(primitive); } catch (error) { threw = error instanceof TypeError; }
  check(threw);
}
for (var ordinary of [{}, [], function () {}, Object.create(null), Error.prototype, TypeError.prototype]) {
  check(get.call(ordinary) === undefined);
}
for (var constructor of [Error, EvalError, RangeError, ReferenceError, SyntaxError, TypeError, URIError, AggregateError, SuppressedError]) {
  var conversions = 0;
  var message = {toString() { ++conversions; return 'captured'; }};
  var args = constructor === AggregateError ? [[], message] : constructor === SuppressedError ? [1, 2, message] : [message];
  var error = Reflect.construct(constructor, args);
  check(conversions === 1 && !Object.hasOwn(error, 'stack'));
  var captured = get.call(error);
  check(typeof captured === 'string' && captured === error.stack);
  error.message = 'changed';
  error.name = 'changed';
  check(get.call(error) === captured);
  error.stack = 'replacement';
  check(error.stack === 'replacement' && get.call(error) === captured);
  var own = Object.getOwnPropertyDescriptor(error, 'stack');
  check(own.writable && own.enumerable && own.configurable);
  delete error.stack;
  check(error.stack === captured && !Object.hasOwn(error, 'stack'));
  check(get.call(Object.create(error)) === undefined);
  var reads = 0;
  var proxy = new Proxy(error, {get() { ++reads; throw 9; }});
  check(get.call(proxy) === undefined && reads === 0);
  var revocable = Proxy.revocable(error, {});
  revocable.revoke();
  check(get.call(revocable.proxy) === undefined);
  function ForeignTarget() {}
  var foreign = Reflect.construct(constructor, args, ForeignTarget);
  check(typeof get.call(foreign) === 'string' && foreign.stack === undefined);
}
class Derived extends TypeError {}
check(typeof get.call(new Derived()) === 'string');
try { null.x; } catch (error) { check(typeof get.call(error) === 'string' && !Object.hasOwn(error, 'stack')); }
