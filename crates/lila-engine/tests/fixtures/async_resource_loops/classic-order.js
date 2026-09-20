const events = [];
function acquire(name) {
  events.push('a' + name);
  return { [Symbol.dispose]() { events.push('d' + name); } };
}
async function task() {
  await 0;
  let index = 0;
  for (using first = acquire('1'), second = acquire('2'); index < 3; index++) {
    try { events.push('b' + index); if (index === 0) continue; break; }
    finally { events.push('f' + index); }
  }
  for (using empty = acquire('0'); false;) { throw 'unreachable'; }
  events.push('after');
  await 0;
  events.push('resumed');
}
task().then(() => {
  if (events.join(',') !== 'a1,a2,b0,f0,b1,f1,d2,d1,a0,d0,after,resumed') throw events.join(',');
  print('ok');
});
