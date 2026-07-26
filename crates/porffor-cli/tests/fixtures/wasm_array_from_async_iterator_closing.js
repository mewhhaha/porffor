const events = [];

const asyncMapperError = { name: "async mapper" };
let asyncIndex = 0;
const asyncSource = {
  [Symbol.asyncIterator]() {
    return this;
  },
  next() {
    return Promise.resolve(
      asyncIndex++ === 0
        ? { done: false, value: 1 }
        : { done: true, value: undefined },
    );
  },
  return() {
    events.push("async:return");
    return Promise.resolve().then(function () {
      events.push("async:cleanup");
      return { done: true };
    });
  },
};

Array.fromAsync(asyncSource, function () {
  throw asyncMapperError;
}).then(
  function () {
    events.push("async:fulfilled");
  },
  function (error) {
    events.push("async:error:" + (error === asyncMapperError));

    const syncMapperError = { name: "sync mapper" };
    let syncIndex = 0;
    const syncSource = {
      [Symbol.iterator]() {
        return this;
      },
      next() {
        return syncIndex++ === 0
          ? { done: false, value: 2 }
          : { done: true, value: undefined };
      },
      return() {
        events.push("sync:return");
        return {
          done: true,
          value: {
            then(resolve) {
              events.push("sync:cleanup");
              resolve(undefined);
            },
          },
        };
      },
    };

    return Array.fromAsync(syncSource, function () {
      return Promise.reject(syncMapperError);
    }).then(
      function () {
        events.push("sync:fulfilled");
      },
      function (error) {
        events.push("sync:error:" + (error === syncMapperError));

        const yieldedValueError = { name: "yielded value" };
        let yielded = false;
        const rejectingSyncSource = {
          [Symbol.iterator]() {
            return this;
          },
          next() {
            if (yielded) {
              return { done: true, value: undefined };
            }
            yielded = true;
            return {
              done: false,
              value: {
                then(_resolve, reject) {
                  events.push("value:then");
                  reject(yieldedValueError);
                },
              },
            };
          },
          return() {
            events.push("value:return");
            return 1;
          },
        };

        return Array.fromAsync(rejectingSyncSource).then(
          function () {
            events.push("value:fulfilled");
          },
          function (error) {
            events.push("value:error:" + (error === yieldedValueError));

            let propertyClosed = false;
            let propertyIndex = 0;
            const propertySource = {
              [Symbol.asyncIterator]() {
                return this;
              },
              next() {
                return Promise.resolve(
                  propertyIndex++ === 0
                    ? { done: false, value: 3 }
                    : { done: true, value: undefined },
                );
              },
              return() {
                propertyClosed = true;
                events.push("property:return");
                return Promise.resolve({ done: true });
              },
            };
            function Unsettable() {
              Object.defineProperty(this, "0", {
                configurable: false,
                value: 0,
              });
            }

            return Array.fromAsync.call(Unsettable, propertySource).then(
              function () {
                events.push("property:fulfilled");
              },
              function (error) {
                events.push(
                  "property:error:" +
                    (error instanceof TypeError) +
                    ":" +
                    propertyClosed,
                );
              },
            );
          },
        );
      },
    );
  },
).then(function () {
  print("array-from-async-closing:" + events.join(","));
});

0;
