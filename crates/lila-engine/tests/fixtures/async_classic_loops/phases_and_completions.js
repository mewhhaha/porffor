function check(condition, label) { if (!condition) throw label; }
var trace = [];
async function phase(label, value) { trace.push(label); await 0; return value; }

async function phases() {
  var reads = [];
  for (let index = await phase('init', 0);
       await phase('test:' + index, index < 2);
       index = await phase('update:' + index, index + 1)) {
    reads.push(() => index);
    await phase('body:' + index, 0);
    gc();
    if (await (index === 0)) continue;
    trace.push('last');
  }
  check(trace.join(',') === 'init,test:0,body:0,update:0,test:1,body:1,last,update:1,test:2', 'phase-order');
  check(reads[0]() === 0 && reads[1]() === 1, 'fresh-original-for-cells');
  trace = [];
  var index = 0;
  do { await phase('body:' + index, 0); index++; }
  while (await phase('test:' + index, index < 2));
  check(trace.join(',') === 'body:0,test:1,body:1,test:2', 'do-body-before-test');

  var headRead, sawTdz = false;
  for (let [value = await { then(resolve) {
    headRead = () => value;
    try { headRead(); } catch (error) { sawTdz = error instanceof ReferenceError; }
    gc(); resolve(7);
  } }] = []; false;) {}
  check(sawTdz && headRead() === 7, 'pattern-head-tdz-and-original-cell');
}

async function completions() {
  trace = [];
  outer: for (let index = await 0; await (index < 3); index = await (index + 1)) {
    try {
      switch (await index) {
        case 0:
          while (await true) { await 0; continue outer; }
          break;
        case 1:
          try { throw { value: 9 }; }
          catch ({ value }) { trace.push(value); await 0; }
          break;
        default:
          await 0;
          break outer;
      }
    } finally { await 0; trace.push('finally:' + index); }
  }
  check(trace.join(',') === 'finally:0,9,finally:1,finally:2', 'label-targets-through-finally');

  async function rejected(which) {
    var marker = {}, captured;
    try {
      for (let index = await (which === 'init' ? Promise.reject(marker) : 0);
           await (which === 'test' ? Promise.reject(marker) : true);
           index = await Promise.reject(marker)) { await 0; }
    } catch (error) { captured = error; }
    finally { await 0; gc(); }
    check(captured === marker, 'phase-rejection:' + which);
  }
  await rejected('init');
  await rejected('test');
  await rejected('update');

  var adopted = [];
  var result = { get then() {
    adopted.push('get-then');
    return resolve => { adopted.push('call-then'); resolve(29); };
  } };
  async function returning() {
    try { for (let index = await 0; await true;) { await 0; return result; } }
    finally { await 0; adopted.push('finally'); }
  }
  check(await returning() === 29, 'return-whole-value');
  check(adopted.join(',') === 'finally,get-then,call-then', 'return-adoption-after-finally');
}

async function run() { await phases(); await completions(); }
run().then(() => print('async-classic-loops:ok'), error => { print(error); throw error; });
