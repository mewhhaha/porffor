globalThis.sharedCount = 0;
globalThis.invalidBody = false;
const events = ['entry'];
let direct;
try { direct = import('./invalid.js'); } catch (_) { throw 'synchronous import throw'; }
const checks = [direct.then(() => { throw 'direct syntax fulfilled'; }, error => {
  if (!(error instanceof SyntaxError)) throw 'wrong direct error';
  events.push('direct');
})];
Promise.resolve().then(() => events.push('tick'));
for (const pending of [import('./transitive.js'), import('./link.js'), import('./missing.js')]) {
  checks.push(pending.then(() => { throw 'invalid closure fulfilled'; }, error => {
    if (!(error instanceof SyntaxError) || events.indexOf('tick') === -1)
      throw 'dependency rejection stage';
    events.push('dependency');
  }));
}
checks.push(import('./valid.js').then(ns => {
  if (ns.value !== 7) throw 'valid shared dependency';
}));
Promise.all(checks).then(() => {
  if (events.slice(0, 3).join(',') !== 'entry,direct,tick' ||
      events.length !== 6 || sharedCount !== 1 || invalidBody)
    throw 'dynamic error admission or timing';
  print('load and dependency rejection stages');
});
true;
