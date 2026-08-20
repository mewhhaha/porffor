async function* values(limit) {
  for (let index = 0; index < limit; index++) {
    yield Promise.resolve(index * 2);
  }
  yield 9;
}

const empty = values(0);
const singleton = values(1);
const multiple = values(3);
const updateError = {};

function throwFromUpdate() {
  throw updateError;
}

async function* failsDuringUpdate() {
  for (let index = 0; index < 2; throwFromUpdate()) {
    yield index;
  }
}

const failing = failsDuringUpdate();

async function* observesFreshLoopTdz() {
  for (let iteration = 0; iteration < 2; iteration++) {
    let observed;
    try {
      observed = later;
    } catch (error) {
      observed = "tdz";
    }
    yield observed;
    let later = iteration;
  }
}

let retainedAfterResume = false;

async function* retainsLoopLexicalAcrossResume() {
  for (let iteration = 0; iteration < 1; iteration++) {
    let value = 7;
    yield value;
    retainedAfterResume = value === 7;
  }
}

const freshTdz = observesFreshLoopTdz();
const retainedLexical = retainsLoopLexicalAcrossResume();

Promise.all([
  empty.next(),
  empty.next(),
  singleton.next(),
  singleton.next(),
  singleton.next(),
  multiple.next(),
  multiple.next(),
  multiple.next(),
  multiple.next(),
  multiple.next(),
  failing.next(),
  failing.next().catch(function (error) {
    return error === updateError;
  }),
  freshTdz.next(),
  freshTdz.next(),
  freshTdz.next(),
  retainedLexical.next(),
  retainedLexical.next(),
]).then(function (results) {
  print(
    "async-generator-resumable-loop:" +
      results[0].value +
      ":" +
      results[0].done +
      ":" +
      results[1].done +
      ":" +
      results[2].value +
      ":" +
      results[2].done +
      ":" +
      results[3].value +
      ":" +
      results[3].done +
      ":" +
      results[4].done +
      ":" +
      results[5].value +
      ":" +
      results[6].value +
      ":" +
      results[7].value +
      ":" +
      results[8].value +
      ":" +
      results[8].done +
      ":" +
      results[9].done +
      ":" +
      results[10].value +
      ":" +
      results[10].done +
      ":" +
      results[11] +
      ":" +
      results[12].value +
      ":" +
      results[12].done +
      ":" +
      results[13].value +
      ":" +
      results[13].done +
      ":" +
      results[14].done +
      ":" +
      results[15].value +
      ":" +
      results[15].done +
      ":" +
      results[16].done +
      ":" +
      retainedAfterResume
  );
});

0;
