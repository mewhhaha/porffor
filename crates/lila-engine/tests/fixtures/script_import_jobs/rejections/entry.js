globalThis.objectCalls = 0;
globalThis.undefinedCalls = 0;
globalThis.reason = { marker: 7, toString() { throw 'rejection was coerced'; } };
async function objectRejection() {
  const first = import('./object.js');
  const second = import('./object.js');
  if (first === second) throw 'reused object import promise';
  let observed;
  let catches = 0;
  try { await first; } catch (error) { observed = error; catches++; }
  try { await second; } catch (error) { if (error !== observed) throw 'concurrent rejection identity'; catches++; }
  try { await import('./object.js'); } catch (error) { if (error !== observed) throw 'cached rejection identity'; catches++; }
  if (catches !== 3 || observed !== reason || objectCalls !== 1) throw 'object rejection cache';
}
async function undefinedRejection() {
  const first = import('./undefined.js');
  const second = import('./undefined.js');
  if (first === second) throw 'reused undefined import promise';
  let catches = 0;
  try { await first; } catch (error) { if (error !== undefined) throw 'first undefined replaced'; catches++; }
  try { await second; } catch (error) { if (error !== undefined) throw 'second undefined replaced'; catches++; }
  try { await import('./undefined.js'); } catch (error) { if (error !== undefined) throw 'cached undefined replaced'; catches++; }
  if (catches !== 3 || undefinedCalls !== 1) throw 'undefined rejection cache';
}
Promise.all([objectRejection(), undefinedRejection()]).then(() => print('cached arbitrary rejections'));
print('rejections queued');
true;
