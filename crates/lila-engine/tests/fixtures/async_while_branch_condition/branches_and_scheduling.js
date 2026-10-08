const trace = [];
let skippedOperands = 0;
let skippedThen = 0;
let skippedJobs = 0;
function skipped() {
  skippedOperands++;
  Promise.resolve().then(() => { skippedJobs++; });
  return {get then() { skippedThen++; throw 'skipped then getter'; }};
}
function chosen(n) {
  trace.push('chosen' + n);
  return {get then() {
    trace.push('then' + n);
    return function(resolve) {
      Promise.resolve().then(() => {
        trace.push('resolve' + n);
        resolve(true);
      });
    };
  }};
}
async function run() {
  let n = 0;
  const readers = [];
  while ((trace.push('head' + n), ++n <= 3) &&
         (n === 2 ? true : (null ?? (false || await chosen(n))))) {
    const value = n;
    readers.push(() => value);
    try {
      trace.push('body' + n);
      if (n === 2) continue;
    } finally {
      trace.push('finally' + n);
    }
  }
  trace.push('done' + n);
  if (n !== 4 || readers.length !== 3 ||
      readers[0]() !== 1 || readers[1]() !== 2 || readers[2]() !== 3) {
    throw 'back edge or eager body environment';
  }
  while (false && await skipped()) { throw 'false and entered'; }
  while (true || await skipped()) { trace.push('or-skip'); break; }
  while (0 ?? await skipped()) { throw 'nonnullish zero entered'; }
  await 0;
  trace.push('post');
  if (skippedOperands !== 0 || skippedThen !== 0 || skippedJobs !== 0) {
    throw 'skipped branch scheduled';
  }
  if (trace.join(',') !== 'head0,chosen1,then1,caller,resolve1,body1,finally1,head1,body2,finally2,head2,chosen3,then3,resolve3,body3,finally3,head3,done4,or-skip,post') {
    throw 'branch scheduling or repeated condition effects';
  }
}
run().then(() => print('while-branches:ok'), error => print('unexpected:' + error));
trace.push('caller');
