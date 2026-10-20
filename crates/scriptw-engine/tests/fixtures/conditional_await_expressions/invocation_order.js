const trace = [];
function record(name, value) { trace.push(name); return value; }
function skipped() { throw 'unchosen branch ran'; }
async function run() {
  const object = {base: 10};
  const original = function(first, second, third) {
    if (this !== object) throw 'receiver changed';
    trace.push('call'); return first + second + third + this.base;
  };
  Object.defineProperty(object, 'method', {get() { trace.push('get'); return original; }, configurable: true});
  const thenable = {get then() {
    trace.push('then');
    Object.defineProperty(object, 'method', {value() { throw 'replacement called'; }, configurable: true});
    return resolve => resolve(2);
  }};
  const value = object.method(record('first', 1),
    record('condition', true) ? await thenable : await skipped(), record('last', 3));
  if (value !== 16 || trace.join(',') !== 'get,first,condition,then,caller,last,call') throw 'method order';
  trace.length = 0;
  function C(value) { trace.push('construct'); this.value = value; }
  const constructor = new C(record('new-test', false) ? await skipped() : await record('new-value', 4));
  if (constructor.value !== 4 || trace.join(',') !== 'new-test,new-value,construct') throw 'constructor order';
  trace.length = 0;
  const tag = {base: 20, method(strings, first, second) {
    if (this !== tag || strings[0] !== '') throw 'tag reference';
    trace.push('tag'); return this.base + first + second;
  }};
  const tagged = tag.method`${record('tag-test', true) ? await record('tag-value', 5) : skipped()}${record('tag-last', 6)}`;
  if (tagged !== 31 || trace.join(',') !== 'tag-test,tag-value,tag-last,tag') throw 'tag order';
}
run().then(() => print('invocation:ok'), error => print('unexpected:' + error));
trace.push('caller');
