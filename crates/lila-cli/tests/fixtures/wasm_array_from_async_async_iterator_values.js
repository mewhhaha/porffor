const promisedValue = Promise.resolve(7);

function asyncSource() {
  let finished = false;
  return {
    [Symbol.asyncIterator]() {
      return this;
    },
    next() {
      if (finished) return Promise.resolve({ done: true });
      finished = true;
      return Promise.resolve({ done: false, value: promisedValue });
    },
  };
}

Array.fromAsync(asyncSource()).then(function (unmapped) {
  let mapperReceivedPromise = false;
  let mapperAwaitCount = 0;

  return Array.fromAsync(asyncSource(), function (value, index) {
    mapperReceivedPromise = value === promisedValue;
    return {
      then(resolve) {
        mapperAwaitCount = mapperAwaitCount + 1;
        resolve(index + 9);
      },
    };
  }).then(function (mapped) {
    print(
      "array-from-async-async-iterator-values:" +
        (unmapped[0] === promisedValue) +
        ":" +
        mapperReceivedPromise +
        ":" +
        mapperAwaitCount +
        ":" +
        mapped.join(","),
    );
  });
});

0;
