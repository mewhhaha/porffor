globalThis.unreachedCalls = 0;
if (false) {
  import('./never.js');
  import('./invalid.js');
}
function neverCalled() { return import('./never.js'); }
if (unreachedCalls !== 0) throw 'uncalled module evaluated';
print('unreached targets stayed idle');
true;
