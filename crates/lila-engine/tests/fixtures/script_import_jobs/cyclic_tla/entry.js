globalThis.events = [];
const result = import('./a.js');
events.push('entry');
result.then(ns => {
  if (ns.a !== 1 || ns.sum() !== 3 ||
      events.join(',') !== 'entry,B start,B end,A start,A end') throw events.join(',');
  print('cyclic TLA graph completed');
});
true;
