function check(value, label) { if (!value) throw label; }
let events = [];
const whole = {tag: 'whole'};
async function take(value, label) { events.push(label); gc(); return value; }
class Private {
  #value = 3;
  static async compound(value) { return (await take(value, 'base')).#value += await take(4, 'rhs'); }
  static async write(value) { return (await take(value, 'base')).#value = await take(9, 'rhs'); }
  static async update(value) { return (await take(value, 'base')).#value++; }
  static async read(value) { return (await take(value, 'base')).#value; }
  static async brand(value) { return #value in await take(value, 'brand'); }
}
const original = {get x() { events.push('get'); return 5; }, set x(value) { events.push('set:' + value); this.saved = value; }};
class Super extends Object {
  async compound(key) { return super[await take(key, 'key')] += await this.change(); }
  async change() { events.push('rhs'); Object.setPrototypeOf(Super.prototype, {set x(value) { throw 'new super base'; }}); gc(); return 2; }
  async write(key) { return super[key] = await take(11, 'rhs'); }
  async update(key) { return super[await take(key, 'key')]++; }
  async remove(key) { return delete super[await take(key, 'key')]; }
}
async function run() {
  const target = new Proxy({x: 2}, {
    get(object, name, receiver) { events.push('get'); return Reflect.get(object, name, receiver); },
    set(object, name, value, receiver) { events.push('set:' + value); return Reflect.set(object, name, value, receiver); }
  });
  const rawKey = {[Symbol.toPrimitive]() { events.push('convert'); return 'x'; }};
  check(await (async function() { return (await take(target, 'base'))[await take(rawKey, 'key')] += await take(6, 'rhs'); })() === 8 && events.join(',') === 'base,key,convert,get,rhs,set:8', 'compound owns Get before RHS');
  events = [];
  check(await (async function() { return target[rawKey] = await take(4, 'rhs'); })() === 4 && events.join(',') === 'rhs,convert,set:4', 'plain raw key conversion after RHS');
  const owner = new Private(); events = [];
  check(await Private.compound(owner) === 7 && events.join(',') === 'base,rhs', 'private compound');
  events = [];
  check(await Private.update(owner) === 7 && await Private.read(owner) === 8 && await Private.brand(owner) === true, 'private read update and in');
  events = []; let error;
  try { await Private.write({}); } catch (value) { error = value; }
  check(error instanceof TypeError && events.join(',') === 'base,rhs', 'private plain Put brand after RHS');
  events = [];
  try { await Private.compound({}); } catch (value) { error = value; }
  check(error instanceof TypeError && events.join(',') === 'base', 'private compound brand before RHS');
  Object.setPrototypeOf(Super.prototype, original); events = [];
  const superOwner = new Super();
  check(await superOwner.compound(rawKey) === 7 && superOwner.saved === 7 && events.join(',') === 'key,convert,get,rhs,set:7', 'captured super base across RHS mutation');
  Object.setPrototypeOf(Super.prototype, original); events = [];
  check(await superOwner.write(rawKey) === 11 && events.join(',') === 'rhs,convert,set:11', 'super plain raw key');
  events = [];
  check(await superOwner.update(rawKey) === 5 && superOwner.saved === 6 && events.join(',') === 'key,convert,get,set:6', 'super update');
  events = []; let converted = 0;
  const forbidden = {[Symbol.toPrimitive]() { converted++; throw 'Delete must not coerce super key'; }};
  try { await superOwner.remove(forbidden); } catch (value) { error = value; }
  check(error instanceof ReferenceError && converted === 0 && events.join(',') === 'key', 'super Delete original ReferenceError');
  events = [];
  try { await (async function() { return target.x += await Promise.reject(whole); })(); } catch (value) { error = value; }
  check(error === whole && events.join(',') === 'get', 'whole RHS rejection no Put');
}
run().then(() => print('async-reference-operations:ok'), error => { print(error); throw error; });
262;
