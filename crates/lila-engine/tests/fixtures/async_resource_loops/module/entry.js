import { count } from './dependency.js';
const events = [];
await 0;
using outer = { [Symbol.dispose]() { if (events.join(',') !== 'loop,body,iteration,after') throw 'outer disposal order'; print('ok'); } };
for (using resource = { [Symbol.dispose]() { events.push('loop'); } }; false;) {}
for (using resource of [{ [Symbol.dispose]() { events.push('iteration'); } }]) {
  try { events.push('body'); }
  finally { if (count !== 2) throw 'dependency completion'; }
}
await 0;
events.push('after');
