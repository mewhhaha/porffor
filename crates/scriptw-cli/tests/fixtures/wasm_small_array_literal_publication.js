function check(value, message) { if (!value) throw new Error(message); }
let calls = [];
function value(n) { calls.push(n); return n; }
const array = [value(0), , value(2), value(3), , value(5)];
check(calls.join(',') === '0,2,3,5', 'evaluation order');
check(array.length === 6 && !(1 in array) && !(4 in array), 'holes and length');
check(Object.keys(array).join(',') === '0,2,3,5', 'own index order');
check(Object.getOwnPropertyDescriptor(array, '2').value === 2, 'descriptor');
array[1] = 1;
array[4] = 4;
for (let i = 6; i < 40; i++) array[i] = i;
for (let i = 0; i < 40; i++) check(array[i] === i, 'growth lookup ' + i);
delete array[2]; array[2] = 42;
check(array[2] === 42 && Object.keys(array).length === 40, 'delete and reinsert count');
array.length = 3;
check(array.length === 3 && array[2] === 42 && !(3 in array), 'length truncation');
const six = [0,1,2,3,4,5];
const seven = [0,1,2,3,4,5,6];
check(six.join(',') === '0,1,2,3,4,5' && seven[6] === 6, 'resize threshold control');
let setterCalls = 0;
Object.defineProperty(Array.prototype, '0', {set() { setterCalls++; }, configurable: true});
let own;
try { own = [17, 18]; } finally { delete Array.prototype[0]; }
check(own[0] === 17 && setterCalls === 0, 'CreateDataProperty ignores inherited setter');
let stopped = false;
try { [value(6), (() => { throw 19; })(), value(7)]; } catch (error) { stopped = error === 19; }
check(stopped && calls.join(',') === '0,2,3,5,6', 'abrupt element order');
const objects = [{x: 1}, {x: 2}, {x: 3}];
const snapshot = Object.getOwnPropertyDescriptor(objects, '0');
objects[0] = {x: 9};
check(snapshot.value.x === 1 && objects[0].x === 9, 'independent descriptor snapshots');
const retained = [];
for (let i = 0; i < 12000; i++) retained.push([i, {x:i}, i + 1, 'v' + i]);
check(retained[0][1].x === 0 && retained[11999][3] === 'v11999', 'GC roots');
print('small-array publication ok');
