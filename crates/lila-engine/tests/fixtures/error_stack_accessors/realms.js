var other = __lilaCreateRealm().global;
var home = Error.prototype;
var foreignHome = other.Error.prototype;
var own = Object.getOwnPropertyDescriptor(home, 'stack');
var foreign = Object.getOwnPropertyDescriptor(foreignHome, 'stack');
var error = new other.TypeError('foreign');
check(typeof own.get.call(error) === 'string' && own.get.call(error) === foreign.get.call(error));
for (var primitive of [null, undefined, 1]) {
  var caught = false;
  try { foreign.get.call(primitive); } catch (error) { caught = error instanceof other.TypeError && !(error instanceof TypeError); }
  check(caught);
}
var caught = false;
try { foreign.set.call(foreignHome, 'home'); } catch (error) { caught = error instanceof other.TypeError; }
check(caught);
var object = {};
foreign.set.call(object, 'foreign');
check(object.stack === 'foreign');
var caught = false;
try { foreign.set.call(new Proxy({}, {defineProperty() { return false; }}), 'reject'); }
catch (error) { caught = error instanceof other.TypeError && !(error instanceof TypeError); }
check(caught);
var caught = false;
try { own.set.call(foreignHome, 'nested'); }
catch (error) { caught = error instanceof other.TypeError && !(error instanceof TypeError); }
check(caught);
var reads = 0;
var proxy = new Proxy(home, {
  getOwnPropertyDescriptor(target, key) { ++reads; return undefined; },
  defineProperty(target, key, descriptor) { ++reads; return true; }
});
own.set.call(proxy, 'proxy');
check(reads === 2 && !Object.hasOwn(proxy, 'stack'));
var getter = foreign.get;
var setter = foreign.set;
other.TypeError = function replacement() {};
other.Error = function replacement() {};
var caught = false;
try { setter.call(foreignHome, 0); } catch (error) { caught = Object.getPrototypeOf(error) !== TypeError.prototype; }
check(caught && typeof getter.call(error) === 'string');
