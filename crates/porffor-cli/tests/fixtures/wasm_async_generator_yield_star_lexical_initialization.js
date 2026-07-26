async function* constDelegate() {
  const completion = yield* {
    [Symbol.asyncIterator]() {
      return {
        next() {
          let observedTdz = false;
          try {
            completion;
          } catch (error) {
            observedTdz = error.constructor === ReferenceError;
          }
          return { done: true, value: observedTdz ? 11 : -1 };
        },
      };
    },
  };
  return completion;
}

async function* letDelegate() {
  let completion = yield* {
    [Symbol.asyncIterator]() {
      return {
        next() {
          return { done: true, value: 13 };
        },
      };
    },
  };
  return completion;
}

Promise.all([constDelegate().next(), letDelegate().next()]).then(function (results) {
  print(
    "async-generator-yield-star-lexical-initialization:" +
      results[0].value +
      ":" +
      results[1].value
  );
});

0;
