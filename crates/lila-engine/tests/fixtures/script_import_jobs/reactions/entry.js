globalThis.events = [];
const result = import('./target.js');
events.push('entry');
Promise.resolve().then(() => events.push('tick1')).then(() => events.push('tick2')).then(() => {
  events.push('tick3');
  if (events.join(',') !== 'entry,body,tick1,tick2,settled,tick3') throw events.join(',');
  print('second reaction');
});
result.then(ns => {
  if (ns.value !== 7) throw 'wrong module value';
  events.push('settled');
}, error => {
  if (error !== 7) throw 'wrong rejection value';
  events.push('settled');
});
true;
