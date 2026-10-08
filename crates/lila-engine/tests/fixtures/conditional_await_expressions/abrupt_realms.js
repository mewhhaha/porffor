const foreign = __lilaCreateRealm().global;
const foreignType = foreign.TypeError.prototype;
const convert = foreign.BigInt;
const marker = new foreign.TypeError('marker');
foreign.TypeError = null;
async function run() {
  const trace = [];
  function skipped() { throw 'unchosen branch'; }
  try {
    const result = true ? await Promise.reject(marker) : await skipped();
    trace.push('wrong write:' + result);
  } catch (error) {
    if (error !== marker || Object.getPrototypeOf(error) !== foreignType) throw 'rejection identity';
    trace.push('caught');
  } finally { await 0; trace.push('finally'); }
  try { const result = false ? await skipped() : await convert(undefined); }
  catch (error) {
    if (Object.getPrototypeOf(error) !== foreignType) throw 'synchronous branch Realm';
    trace.push('foreign');
  }
  const value = false ? await skipped() : await 6;
  if (value !== 6 || trace.join(',') !== 'caught,finally,foreign') throw 'abrupt join';
}
run().then(() => print('abrupt:ok'), error => print('unexpected:' + error));
