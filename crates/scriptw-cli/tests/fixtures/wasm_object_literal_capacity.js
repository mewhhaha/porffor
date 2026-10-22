function check(value, message) { if (!value) throw new Error(message); }
const events = [];
function visit(name, value) { events.push(name); return value; }
const proto = { inherited: 41 };
const object = {
  __proto__: visit('proto', proto),
  a: visit('a', 1), b: 2, c: 3, d: 4, e: 5, f: 6, g: 7, h: 8,
  a: visit('duplicate', 9),
  get pair() { return this.a; },
  set pair(value) { this.a = value; },
  method() { return super.inherited + this.a; }
};
check(events.join(',') === 'proto,a,duplicate', 'evaluation order');
check(Object.keys(object).join(',') === 'a,b,c,d,e,f,g,h,pair,method', 'first insertion order');
check(object.method() === 50 && object.pair === 9, 'prototype and method home object');
object.pair = 11;
check(object.a === 11, 'merged accessor');
const snapshot = Object.getOwnPropertyDescriptor(object, 'a');
delete object.b;
object.b = 20;
for (let i = 0; i < 40; i++) object['extra' + i] = i;
check(snapshot.value === 11 && Object.keys(object)[9] === 'b', 'snapshot and delete/reinsert');
check(object.extra39 === 39 && object.method() === 52, 'later storage growth');
Object.freeze(object);
check(Object.isFrozen(object), 'freeze after growth');
const symbol = Symbol('dynamic');
const dynamic = { a: 1, b: 2, c: 3, d: 4, e: 5, [symbol]: 6, ...{a: 7, z: 8} };
check(dynamic.a === 7 && dynamic[symbol] === 6 && dynamic.z === 8, 'dynamic fallback');
const rows = [];
for (let i = 0; i < 12000; i++) rows.push({a:i,b:1,c:2,d:3,e:4,f:5,g:6,h:7,i:8});
check(rows[0].a === 0 && rows[11999].a === 11999 && rows[4096].i === 8, 'retained roots');
print('object-literal capacity ok');
