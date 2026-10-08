function check(condition, label) { if (!condition) throw label; }
var whole = { marker: 81 }, closeError = { marker: 82 };
whole[Symbol.toPrimitive] = function () { throw 'converted-whole'; };

function iterator(log, throws) {
  return { [Symbol.iterator]: function () { return this; }, next: function () { log.push('next'); return {done:false,value:undefined}; }, return: function () { log.push('close'); if (throws) throw closeError; return {}; } };
}

function generators() {
  var log = [];
  function* values(input) { try { throw input; } catch([received=yield ()=>received]) { log.push('body'); yield ()=>received; } finally { log.push('finally'); yield 'finally'; } }
  var current = values(iterator(log, false)), pending = current.next().value;
  try { pending(); throw 'catch-default-initialized-early'; } catch(error) { check(error instanceof ReferenceError, 'generator-catch-default-tdz'); }
  gc(); var reader = current.next(whole).value;
  check(reader() === whole && pending() === whole && log.join(',') === 'next,close,body', 'generator-catch-cell-and-close-before-body');
  check(current.next().value === 'finally' && current.next().done, 'generator-catch-finalizer-completes');gc();
  check(reader() === whole, 'generator-catch-cell-escapes-completion');
  log = []; current = values(iterator(log, true));current.next();
  check(current.throw(whole).value === 'finally' && log.join(',') === 'next,close,finally', 'generator-injected-throw-closes-before-outer-finalizer');
  try { current.next(); throw 'lost-injected-whole-throw'; } catch(error) { check(error === whole, 'generator-whole-throw-wins-close-error'); }
  log = []; current = values(iterator(log, false));current.next();
  check(current.return(whole).value === 'finally', 'generator-return-enters-original-finalizer');
  var terminal = current.next();check(terminal.done && terminal.value === whole && log.join(',') === 'next,close,finally', 'generator-return-preserves-whole-through-close');
}

async function asynchronous() {
  var log = [], resume, pending;
  var waiting = new Promise(function(resolve) { resume=resolve; });
  async function values(input) { try { throw input; } catch([received=await (pending=()=>received,waiting)]) { let body='body';log.push(body);return ()=>[body,received]; } finally { await 0;log.push('finally'); } }
  var completion = values(iterator(log,false));
  try { pending();throw 'async-catch-default-initialized-early'; } catch(error) { check(error instanceof ReferenceError, 'async-catch-default-tdz'); }
  gc();resume(whole);var reader = await completion;
  check(reader()[0] === 'body' && reader()[1] === whole && pending() === whole && log.join(',') === 'next,close,body,finally', 'async-original-parameter-body-and-close');gc();
  check(reader()[1] === whole, 'async-catch-cell-escapes-completion');
  log=[];
  async function rejected(input) { try { throw input; } catch([received=await Promise.reject(whole)]) { throw 'entered-rejected-catch-body'; } finally { await 0;log.push('finally'); } }
  try { await rejected(iterator(log,true));throw 'lost-rejected-catch-default'; } catch(error) { check(error === whole, 'async-catch-default-whole-rejection-wins-close-error'); }
  check(log.join(',') === 'next,close,finally', 'async-rejected-catch-default-closes-before-finalizer');
  var gets=0;
  async function computed(input) { try { throw input; } catch({[await 'missing']:received=await whole}) { return ()=>received; } }
  reader = await computed({get missing(){gets++;return undefined;}});
  check(gets === 1 && reader() === whole, 'async-catch-computed-key-get-before-lazy-default');
}
generators();
asynchronous().then(function(){print('resumable-catch-patterns:ok');},function(error){print(error);throw error;});
