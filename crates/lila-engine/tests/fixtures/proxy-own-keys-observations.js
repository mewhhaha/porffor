var log = [];
var base = {};
Object.defineProperty(base, 'a', {value: 1});
Object.defineProperty(base, 'b', {value: 2});
var target = new Proxy(base, {
  isExtensible(t) { log.push('extensible'); return Reflect.isExtensible(t); },
  ownKeys(t) { log.push('keys'); return Reflect.ownKeys(t); },
  getOwnPropertyDescriptor(t, key) { log.push('desc:' + key); return Reflect.getOwnPropertyDescriptor(t, key); }
});
try { Reflect.ownKeys(new Proxy(target, {ownKeys() { log.push('outer'); return []; }})); }
catch (error) { log.push(error instanceof TypeError ? 'TypeError' : 'other'); }
print(log.join(' '));
var sentinel = {};
var keys = ['a'];
Object.defineProperty(keys, '0', {get() { throw sentinel; }});
try { Reflect.ownKeys(new Proxy({}, {ownKeys() { return keys; }})); }
catch (error) { print(error === sentinel); }
var seen = [];
var duplicate = {length: 3, get 0() { seen.push('0'); return 'a'; },
  get 1() { seen.push('1'); return 'a'; }, get 2() { seen.push('2'); return 'b'; }};
try { Reflect.ownKeys(new Proxy({}, {ownKeys() { return duplicate; }})); }
catch (error) { print(seen.join(' '), error instanceof TypeError); }
var typed = new Uint8Array(2);
Object.preventExtensions(typed);
try { Reflect.ownKeys(new Proxy(typed, {ownKeys() { return []; }})); }
catch (error) { print(error instanceof TypeError); }
