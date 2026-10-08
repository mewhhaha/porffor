function check(value, label) { if (!value) throw label; }
let events = [];
const whole = {tag: 'whole'};
async function key(value) { events.push('key'); gc(); return value; }
async function argument() { events.push('argument'); gc(); return 7; }
class Private {
  get #method() {
    events.push('private-get');
    return function(value) { 'use strict'; check(this instanceof Private && value === 7, 'private receiver'); return 29; };
  }
  static async call(value) { return value?.#method(await argument()); }
  static async grouped(value) { return (value?.[await key('self')].#method)(await argument()); }
  static async brand(value) { return value?.#method(await argument()); }
}
const prototype = {
  get method() {
    events.push('super-get');
    const owner = this;
    return function(value) { 'use strict'; check(this === owner && value === 7, 'super receiver'); return 31; };
  }
};
class Super extends Object {
  async call() { return super[await key('method')]?.(await this.change()); }
  async change() {
    events.push('change');
    Object.setPrototypeOf(Super.prototype, {method() { throw 'replacement callee'; }});
    gc(); return 7;
  }
}
Object.setPrototypeOf(Super.prototype, prototype);
async function run() {
  events = [];
  check(await Private.call(null) === undefined && events.length === 0, 'null skips private brand and Await');
  const owner = new Private(); owner.self = owner;
  check(await Private.call(owner) === 29 && events.join(',') === 'private-get,argument', 'selected private call');
  events = [];
  check(await Private.grouped(owner) === 29 && events.join(',') === 'key,private-get,argument', 'grouping retains private receiver');
  events = [];
  let error;
  try { await Private.brand({}); } catch (value) { error = value; }
  check(error instanceof TypeError && events.length === 0, 'private brand failure precedes arguments');
  check(await new Super().call() === 31 && events.join(',') === 'key,super-get,change', 'original super callee and receiver');

  events = [];
  let conversions = 0, gets = 0, deletes = 0;
  const rawKey = {[Symbol.toPrimitive]() { conversions++; events.push('convert'); return 'value'; }};
  const target = new Proxy({value: 1}, {
    get(object, name, receiver) { gets++; return Reflect.get(object, name, receiver); },
    deleteProperty(object, name) { deletes++; events.push('delete'); return Reflect.deleteProperty(object, name); }
  });
  async function remove(value) { return delete value?.[await key(rawKey)]; }
  check(await remove(null) === true && events.length === 0 && conversions === 0, 'skipped Delete returns true without Await');
  check(await remove(target) === true && gets === 0 && deletes === 1 && conversions === 1 && events.join(',') === 'key,convert,delete', 'Delete owns Reference without Get');
  events = [];
  check(await (async function() { return delete (await target)?.[rawKey]; })() === true && gets === 0 && conversions === 2 && deletes === 2, 'target-only Await Delete');
  events = [];
  const holder = new Proxy({child: target}, {get(object, name, receiver) { events.push('holder-get'); return Reflect.get(object, name, receiver); }});
  check(await (async function() { return delete holder?.[await key('child')][await key(rawKey)]; })() === true && events.join(',') === 'key,holder-get,key,convert,delete', 'earlier Get before later Delete key');
  const falseDelete = new Proxy({}, {deleteProperty() { return false; }});
  error = undefined;
  try { await (async function() { 'use strict'; return delete falseDelete?.[await key('x')]; })(); } catch (value) { error = value; }
  check(error instanceof TypeError, 'strict false Delete');
  let calls = 0;
  check(await (async function() { return delete ({method(value) { calls++; return value; }})?.method(await 1); })() === true && calls === 1, 'terminal Call deletes Value');
  error = undefined;
  try { await (async function() { return delete target?.[await Promise.reject(whole)]; })(); } catch (value) { error = value; }
  check(error === whole && deletes === 3, 'rejected key prevents Delete');
}
run().then(() => print('optional-references:ok'), error => { print(error); throw error; });
262;
