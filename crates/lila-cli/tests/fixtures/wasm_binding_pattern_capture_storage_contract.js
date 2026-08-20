let { rootPattern } = { rootPattern: 2 };
let [rootArrayPattern] = [3];

function capturesBlockObjectPattern() {
  let value = 1;
  {
    let { value } = { value: 2 };
    return (() => value)() === 2;
  }
}

function capturesBlockArrayPattern() {
  let value = 1;
  {
    let [value] = [2];
    return (() => value)() === 2;
  }
}

function capturesCatchObjectPattern() {
  let value = 1;
  try {
    throw { value: 2 };
  } catch ({ value }) {
    return (() => value)() === 2;
  }
}

function capturesForOfObjectPattern() {
  let value = 1;
  let read;
  for (let { value } of [{ value: 2 }]) {
    read = () => value;
    break;
  }
  return read() === 2;
}

function capturesForOfObjectPatterns() {
  let reads = [];
  for (let { x, y } of [{ x: 2, y: 3 }, { x: 4, y: 5 }]) {
    reads.push(() => x * 10 + y);
  }
  return reads[0]() === 23 && reads[1]() === 45;
}

function capturesGenericForOfObjectPatterns() {
  var position = 0;
  var iterator = {
    next: function() {
      position += 1;
      if (position === 1) return { done: false, value: { x: 6, y: 7 } };
      if (position === 2) return { done: false, value: { x: 8, y: 9 } };
      return { done: true };
    },
  };
  iterator[Symbol.iterator] = function() { return iterator; };

  let reads = [];
  for (let { x, y } of iterator) {
    reads.push(() => x * 10 + y);
  }
  return reads[0]() === 67 && reads[1]() === 89;
}

function capturesVarObjectPattern() {
  var { value } = { value: 2 };
  return () => value;
}

function capturesClassicForHead() {
  let value = 1;
  let read;
  for (let value = 2; value < 3; value++) {
    read = () => value;
    break;
  }
  return read() === 2;
}

function capturesClassicForInitializer() {
  let read;
  for (let value = 2, unused = read = () => value; false; ) {}
  return read() === 2;
}

function capturesLaterClassicForBinding() {
  let read;
  for (let inner = read = () => value, value = 2; false; ) {}
  return read() === 2;
}

function preservesTransitiveScopedAlias() {
  let value = 0;
  {
    let value = 2;
    function middle() {
      function inner() {
        return value;
      }
      return inner();
    }
    return middle() === 2;
  }
}

(() => rootPattern)() === 2 &&
  (() => rootArrayPattern)() === 3 &&
  capturesBlockObjectPattern() &&
  capturesBlockArrayPattern() &&
  capturesCatchObjectPattern() &&
  capturesForOfObjectPattern() &&
  capturesForOfObjectPatterns() &&
  capturesGenericForOfObjectPatterns() &&
  capturesVarObjectPattern()() === 2 &&
  capturesClassicForHead() &&
  capturesClassicForInitializer() &&
  capturesLaterClassicForBinding() &&
  preservesTransitiveScopedAlias();
