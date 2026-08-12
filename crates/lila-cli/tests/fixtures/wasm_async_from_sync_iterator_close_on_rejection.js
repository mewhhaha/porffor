// AsyncFromSyncIteratorContinuation (ECMA-262 27.1.4.4) steps 6.a and 13: the
// wrapped SYNC iterator must be closed when the value wrapper rejects, exactly
// once, and must not be closed when the spec says not to.
//
// # Why this fixture exists next to six test262 cases
//
// Four of the six test262 cases in
// `built-ins/AsyncFromSyncIteratorPrototype` assert `sameValue(returnCount, 1)`.
// That catches 0 and it catches 2, but only for the two shapes those cases
// happen to build. It says nothing about:
//
//   * a `return` that is ABSENT or NOT CALLABLE — IteratorClose's GetMethod
//     step must leave the rejection alone rather than turn it into a TypeError;
//   * a `return` that THROWS — 7.4.9 step 5 ("if completion is a throw
//     completion, return ? completion") fires BEFORE step 6 ("if innerResult is
//     a throw completion, return ? innerResult"), so with a throw completion the
//     ORIGINAL reason always wins and the close's own error is swallowed;
//   * `done === true` (step 12), where NO onRejected exists and the iterator
//     must NOT be closed;
//   * `closeOnRejection === false` — the `%AsyncFromSyncIteratorPrototype%.return`
//     entry, reached here as `asyncGenerator.return()` during a `yield*`. The
//     sync `return` has already been called to produce the result being
//     unwrapped, so closing again is the double close that a `returnCount === 1`
//     assertion on the OTHER paths can never see.
//
// Every case below drives the async-generator delegation path
// (`compile_async_generator_delegation`), which is the path the six test262
// cases use: they all wrap the sync iterable in `async function* () { yield* … }`.
// The two for-await cases in the same node take the loop path in
// `compile_async_for_of_iterator`, which closes on its own and passes already.

const results = [];

function record(name, value) {
  results.push(name + "=" + value);
}

// ------------------------------------------------------------------ step 13
// The iterator result's `value` is an already-rejected promise. The value
// wrapper settles as a rejection, so step 13's closeIterator closure runs
// IteratorClose(syncIteratorRecord, ThrowCompletion(error)) exactly once, and
// the ORIGINAL reason reaches the caller.

let stepThirteenReturns = 0;
const stepThirteenIterable = {
  [Symbol.iterator]() {
    return {
      next() {
        return { value: Promise.reject("A"), done: false };
      },
      return() {
        stepThirteenReturns += 1;
      },
    };
  },
};

// ----------------------------------------------------------------- step 6.a
// `PromiseResolve(%Promise%, value)` itself abrupt-completes, because reading
// the already-a-promise value's `constructor` throws. Step 6.a closes the sync
// iterator and the poison error — not a TypeError, not a close error — is what
// rejects the capability.

let stepSixReturns = 0;
const stepSixIterable = {
  [Symbol.iterator]() {
    return {
      next() {
        const poisoned = Promise.resolve("unreachable");
        Object.defineProperty(poisoned, "constructor", {
          get() {
            throw "B";
          },
        });
        return { value: poisoned, done: false };
      },
      return() {
        stepSixReturns += 1;
      },
    };
  },
};

// ------------------------------------------------------- absent `return`
// GetMethod returns undefined, IteratorClose returns the completion unchanged,
// and nothing throws on the way out.

const absentReturnIterable = {
  [Symbol.iterator]() {
    return {
      next() {
        return { value: Promise.reject("C"), done: false };
      },
    };
  },
};

// ------------------------------------------------- non-callable `return`
// GetMethod throws a TypeError. With a throw completion that TypeError is
// swallowed by 7.4.9 step 5 and the original reason survives.

const uncallableReturnIterable = {
  [Symbol.iterator]() {
    return {
      next() {
        return { value: Promise.reject("D"), done: false };
      },
      return: 42,
    };
  },
};

// ------------------------------------------------------- throwing `return`
// The close runs (so the count is 1) and its error is discarded: the caller
// still sees the promise's rejection reason. Getting this backwards passes the
// three test262 step-13 cases and silently swallows the real reason.

let throwingReturnReturns = 0;
const throwingReturnIterable = {
  [Symbol.iterator]() {
    return {
      next() {
        return { value: Promise.reject("E"), done: false };
      },
      return() {
        throwingReturnReturns += 1;
        throw "E-from-return";
      },
    };
  },
};

// --------------------------------------------------------- step 12: done
// `done` is true, so step 12 installs no onRejected at all. The rejection
// propagates and the iterator is NOT closed.

let doneReturns = 0;
const doneIterable = {
  [Symbol.iterator]() {
    return {
      next() {
        return { value: Promise.reject("F"), done: true };
      },
      return() {
        doneReturns += 1;
      },
    };
  },
};

// ------------------------------------------- closeOnRejection === false
// `asyncGenerator.return()` during a `yield*` routes through
// %AsyncFromSyncIteratorPrototype%.return, which passes closeOnRejection =
// false. `return` has already been called once to produce this result; the
// rejecting value must not call it a second time.

let closeOnRejectionFalseReturns = 0;
const closeOnRejectionFalseIterable = {
  [Symbol.iterator]() {
    return {
      next() {
        return { value: 1, done: false };
      },
      return() {
        closeOnRejectionFalseReturns += 1;
        return { value: Promise.reject("H"), done: false };
      },
    };
  },
};

// The wrappers are declared at the top level, exactly the shape the six
// test262 cases use, so nothing here depends on nested async-generator
// declarations inside an async function body.

async function* stepThirteenGenerator() {
  yield* stepThirteenIterable;
}

async function* stepSixGenerator() {
  yield* stepSixIterable;
}

async function* absentReturnGenerator() {
  yield* absentReturnIterable;
}

async function* uncallableReturnGenerator() {
  yield* uncallableReturnIterable;
}

async function* throwingReturnGenerator() {
  yield* throwingReturnIterable;
}

async function* doneGenerator() {
  yield* doneIterable;
}

async function* closeOnRejectionFalseGenerator() {
  yield* closeOnRejectionFalseIterable;
}

async function firstRejection(thunk) {
  try {
    await thunk();
  } catch (error) {
    return error;
  }
  return "no-rejection";
}

async function main() {
  record(
    "a",
    (await firstRejection(() => stepThirteenGenerator().next())) +
      "/" +
      stepThirteenReturns
  );

  record(
    "b",
    (await firstRejection(() => stepSixGenerator().next())) + "/" + stepSixReturns
  );

  record("c", await firstRejection(() => absentReturnGenerator().next()));

  record("d", await firstRejection(() => uncallableReturnGenerator().next()));

  record(
    "e",
    (await firstRejection(() => throwingReturnGenerator().next())) +
      "/" +
      throwingReturnReturns
  );

  record(
    "f",
    (await firstRejection(() => doneGenerator().next())) + "/" + doneReturns
  );

  const held = closeOnRejectionFalseGenerator();
  await held.next();
  record(
    "h",
    (await firstRejection(() => held.return("ignored"))) +
      "/" +
      closeOnRejectionFalseReturns
  );

  print("async-from-sync-close:" + results.join("|"));
}

main().then(undefined, function (error) {
  print("async-from-sync-close:FAILED:" + error + ":" + results.join("|"));
});

0;
