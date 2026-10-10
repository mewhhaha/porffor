function expectTypeError(action) {
  try { action(); } catch (error) {
    if (Object.getPrototypeOf(error) !== TypeError.prototype) throw 'snapshot native TypeError';
    return;
  }
  throw 'missing snapshot native TypeError';
}
const receiver = {};
const keys = ['x', Symbol('ordinary-descriptor-snapshot')];
let cases = 0;
for (const key of keys) {
  const data = {};
  Object.defineProperty(data, key, {value: 1, writable: true, enumerable: true, configurable: false});
  let dataConversions = 0;
  let dataFields = {
    get value() {
      dataConversions++;
      Object.defineProperty(data, key, {value: 2, writable: false});
      return 1;
    },
    writable: true, enumerable: true, configurable: false
  };
  const dataProxy = new Proxy(data, {getOwnPropertyDescriptor() {return dataFields;}});
  // The target descriptor is acquired before ToPropertyDescriptor calls the
  // value getter. Compatibility must use that independent writable snapshot.
  const beforeDataMutation = Reflect.getOwnPropertyDescriptor(dataProxy, key);
  if (beforeDataMutation.value !== 1 || !beforeDataMutation.writable ||
      !beforeDataMutation.enumerable || beforeDataMutation.configurable ||
      dataConversions !== 1) throw 'completed data descriptor snapshot';
  if (Reflect.get(dataProxy, key, receiver) !== 2 || Reflect.set(dataProxy, key, 3, receiver) ||
      Object.hasOwn(receiver, key)) throw 'fresh data descriptor after conversion mutation';
  dataFields = {value: 1, writable: false, enumerable: true, configurable: false};
  expectTypeError(() => Reflect.getOwnPropertyDescriptor(dataProxy, key));
  dataFields = {value: 2, writable: false, enumerable: true, configurable: false};
  const afterDataMutation = Reflect.getOwnPropertyDescriptor(dataProxy, key);
  if (afterDataMutation.value !== 2 || afterDataMutation.writable) throw 'fresh fixed data descriptor';

  const accessor = {};
  let oldCalls = 0;
  function oldGetter() {oldCalls++; return -1;}
  function oldSetter() {oldCalls++;}
  function newGetter() {
    if (this !== receiver) throw 'fresh getter receiver';
    return 4;
  }
  let setValue = 0;
  function newSetter(value) {
    if (this !== receiver) throw 'fresh setter receiver';
    setValue = value;
  }
  Object.defineProperty(accessor, key, {
    get: oldGetter, set: oldSetter, enumerable: true, configurable: true
  });
  let accessorConversions = 0;
  const accessorFields = {
    enumerable: true,
    get configurable() {
      accessorConversions++;
      Object.defineProperty(accessor, key, {
        get: newGetter, set: newSetter, enumerable: false, configurable: false
      });
      return true;
    },
    get: oldGetter, set: oldSetter
  };
  const accessorProxy = new Proxy(accessor, {getOwnPropertyDescriptor() {return accessorFields;}});
  const beforeAccessorMutation = Object.getOwnPropertyDescriptor(accessorProxy, key);
  if (beforeAccessorMutation.get !== oldGetter || beforeAccessorMutation.set !== oldSetter ||
      !beforeAccessorMutation.enumerable || !beforeAccessorMutation.configurable ||
      Object.hasOwn(beforeAccessorMutation, 'value') || accessorConversions !== 1 || oldCalls !== 0)
    throw 'completed accessor descriptor snapshot';
  if (Reflect.get(accessorProxy, key, receiver) !== 4 ||
      !Reflect.set(accessorProxy, key, 5, receiver) || setValue !== 5 || oldCalls !== 0)
    throw 'fresh accessor descriptor after conversion mutation';
  const afterAccessorMutation = Object.getOwnPropertyDescriptor(accessor, key);
  if (afterAccessorMutation.get !== newGetter || afterAccessorMutation.set !== newSetter ||
      afterAccessorMutation.enumerable || afterAccessorMutation.configurable)
    throw 'fresh fixed accessor descriptor';
  // The second acquisition must see the now fixed descriptor and reject the
  // same configurable trap result, rather than reusing the earlier snapshot.
  expectTypeError(() => Object.getOwnPropertyDescriptor(accessorProxy, key));
  if (accessorConversions !== 2 || oldCalls !== 0) throw 'descriptor acquisition lifecycle';
  cases++;
}
if (cases !== 2) throw 'String and Symbol descriptor mutation census';
print('ordinary-descriptor-snapshot-mutation:ok');
262;
