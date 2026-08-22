// Consumer oracle for a classic `for` whose lexical head owns an asynchronous
// DisposeCapability. There are no explicit Await or Yield expressions: every
// suspension below belongs to the loop's implicit resource finalizer.

function same(actual, expected, label) {
  if (actual !== expected) throw label;
}

function sameTrace(actual, expected, label) {
  same(actual.length, expected.length, label + " length");
  for (let i = 0; i < expected.length; i++) {
    same(actual[i], expected[i], label + " " + i);
  }
}

function asyncResource(label, trace, error) {
  let count = 0;
  let value = { label: label };
  Object.defineProperty(value, Symbol.asyncDispose, {
    get: function () {
      trace.push(label + ":get-async");
      return async function () {
        if (this !== value) throw label + " receiver";
        count++;
        trace.push(label + ":dispose");
        if (error !== undefined) throw error;
      };
    },
  });
  Object.defineProperty(value, Symbol.dispose, {
    get: function () {
      trace.push(label + ":bad-get-sync");
      throw label + " sync fallback read";
    },
  });
  return {
    value: value,
    count: function () {
      return count;
    },
  };
}

function syncFallbackResource(label, trace) {
  let count = 0;
  let thenReads = 0;
  let value = { label: label };
  Object.defineProperty(value, Symbol.asyncDispose, {
    get: function () {
      trace.push(label + ":get-async");
      return undefined;
    },
  });
  Object.defineProperty(value, Symbol.dispose, {
    get: function () {
      trace.push(label + ":get-sync");
      return function () {
        if (this !== value) throw label + " receiver";
        count++;
        trace.push(label + ":dispose");
        return {
          get then() {
            thenReads++;
            trace.push(label + ":bad-then");
            return function () {};
          },
        };
      };
    },
  });
  return {
    value: value,
    count: function () {
      return count;
    },
    thenReads: function () {
      return thenReads;
    },
  };
}

async function normalLoop(trace, direct, fallback) {
  let iteration = 0;
  for (
    await using directBinding = direct.value,
      fallbackBinding = fallback.value;
    iteration < 1;
    iteration++
  ) {
    same(directBinding, direct.value, "normal direct binding");
    same(fallbackBinding, fallback.value, "normal fallback binding");
    same(direct.count(), 0, "normal direct before disposal");
    same(fallback.count(), 0, "normal fallback before disposal");
    trace.push("normal:body");
  }
  trace.push("normal:after");
  return "normal-result";
}

async function breakLoop(trace, held) {
  for (await using binding = held.value; true; ) {
    trace.push("break:body");
    break;
  }
  trace.push("break:after");
}

async function labelledBreakLoop(trace, held) {
  outer: for (await using binding = held.value; true; ) {
    for (let inner = 0; inner < 1; inner++) {
      trace.push("label-break:body");
      break outer;
    }
  }
  trace.push("label-break:after");
}

async function continueLoop(trace, held) {
  let iteration = 0;
  for (await using binding = held.value; iteration < 2; iteration++) {
    trace.push("continue:body:" + iteration + ":" + held.count());
    continue;
  }
  trace.push("continue:after");
}

async function labelledContinueLoop(trace, held) {
  let iteration = 0;
  outer: for (await using binding = held.value; iteration < 2; iteration++) {
    for (let inner = 0; inner < 1; inner++) {
      trace.push("label-continue:body:" + iteration + ":" + held.count());
      continue outer;
    }
  }
  trace.push("label-continue:after");
}

async function returnLoop(trace, held) {
  for (await using binding = held.value; true; ) {
    trace.push("return:body");
    return 17;
  }
}

async function throwLoop(trace, held, error) {
  for (await using binding = held.value; true; ) {
    trace.push("throw:body");
    throw error;
  }
}

async function testAbruptLoop(trace, held, error) {
  function failTest() {
    trace.push("test:abrupt");
    throw error;
  }
  for (await using binding = held.value; failTest(); ) {
    trace.push("test:unreachable");
  }
}

async function updateAbruptLoop(trace, held, error) {
  let iteration = 0;
  function failUpdate() {
    trace.push("update:abrupt");
    throw error;
  }
  for (
    await using binding = held.value;
    iteration < 1;
    (iteration++, failUpdate())
  ) {
    trace.push("update:body");
  }
}

async function laterAcquisitionFailure(trace, invalid) {
  for (
    await using first = {
        get [Symbol.asyncDispose]() {
          let state = "visible";
          try {
            second;
          } catch (error) {
            state = error.name;
          }
          trace.push("acquire:later-tdz:" + state);
          return async function () {
            trace.push("acquire:first-dispose");
          };
        },
      },
      second = invalid;
    false;
  ) {
    trace.push("acquire:unreachable");
  }
}

async function suppressedLoop(trace, first, second, bodyError) {
  for (
    await using firstBinding = first.value, secondBinding = second.value;
    true;
  ) {
    trace.push("suppressed:body");
    throw bodyError;
  }
}

async function captureLoop(trace, held) {
  let run = true;
  let capture;
  let binding = "outer";
  for (await using binding = held.value; run; run = false) {
    capture = function () {
      return binding;
    };
    trace.push("capture:body:" + capture().label + ":" + held.count());
  }
  trace.push(
    "capture:after:" + binding + ":" + capture().label + ":" + held.count()
  );
  return capture();
}

function fulfilled(promise, label, check) {
  return promise.then(check, function (error) {
    throw label + " rejected: " + error;
  });
}

function rejected(promise, expected, label, check) {
  return promise.then(
    function () {
      throw label + " fulfilled";
    },
    function (error) {
      same(error, expected, label + " identity");
      check(error);
    }
  );
}

function main() {
  let normalTrace = [];
  let direct = asyncResource("normal:direct", normalTrace);
  let fallback = syncFallbackResource("normal:fallback", normalTrace);
  return fulfilled(
    normalLoop(normalTrace, direct, fallback),
    "normal loop",
    function (result) {
      same(result, "normal-result", "normal result");
      sameTrace(
        normalTrace,
        [
          "normal:direct:get-async",
          "normal:fallback:get-async",
          "normal:fallback:get-sync",
          "normal:body",
          "normal:fallback:dispose",
          "normal:direct:dispose",
          "normal:after",
        ],
        "normal lifecycle"
      );
      same(direct.count(), 1, "normal direct exactly once");
      same(fallback.count(), 1, "normal fallback exactly once");
      same(fallback.thenReads(), 0, "sync fallback return ignored");
    }
  )
    .then(function () {
      let trace = [];
      let held = asyncResource("break", trace);
      return fulfilled(breakLoop(trace, held), "break loop", function () {
        sameTrace(
          trace,
          ["break:get-async", "break:body", "break:dispose", "break:after"],
          "break lifecycle"
        );
        same(held.count(), 1, "break exactly once");
      });
    })
    .then(function () {
      let trace = [];
      let held = asyncResource("label-break", trace);
      return fulfilled(
        labelledBreakLoop(trace, held),
        "labelled break loop",
        function () {
          sameTrace(
            trace,
            [
              "label-break:get-async",
              "label-break:body",
              "label-break:dispose",
              "label-break:after",
            ],
            "labelled break lifecycle"
          );
          same(held.count(), 1, "labelled break exactly once");
        }
      );
    })
    .then(function () {
      let trace = [];
      let held = asyncResource("continue", trace);
      return fulfilled(continueLoop(trace, held), "continue loop", function () {
        sameTrace(
          trace,
          [
            "continue:get-async",
            "continue:body:0:0",
            "continue:body:1:0",
            "continue:dispose",
            "continue:after",
          ],
          "continue lifecycle"
        );
        same(held.count(), 1, "continue exactly once");
      });
    })
    .then(function () {
      let trace = [];
      let held = asyncResource("label-continue", trace);
      return fulfilled(
        labelledContinueLoop(trace, held),
        "labelled continue loop",
        function () {
          sameTrace(
            trace,
            [
              "label-continue:get-async",
              "label-continue:body:0:0",
              "label-continue:body:1:0",
              "label-continue:dispose",
              "label-continue:after",
            ],
            "labelled continue lifecycle"
          );
          same(held.count(), 1, "labelled continue exactly once");
        }
      );
    })
    .then(function () {
      let trace = [];
      let held = asyncResource("return", trace);
      return fulfilled(returnLoop(trace, held), "return loop", function (value) {
        same(value, 17, "return value");
        sameTrace(
          trace,
          ["return:get-async", "return:body", "return:dispose"],
          "return lifecycle"
        );
      });
    })
    .then(function () {
      let trace = [];
      let held = asyncResource("throw", trace);
      let error = { label: "body error" };
      return rejected(throwLoop(trace, held, error), error, "throw loop", function () {
        sameTrace(
          trace,
          ["throw:get-async", "throw:body", "throw:dispose"],
          "throw lifecycle"
        );
      });
    })
    .then(function () {
      let trace = [];
      let held = asyncResource("test", trace);
      let error = { label: "test error" };
      return rejected(
        testAbruptLoop(trace, held, error),
        error,
        "test abrupt loop",
        function () {
          sameTrace(
            trace,
            ["test:get-async", "test:abrupt", "test:dispose"],
            "test abrupt lifecycle"
          );
        }
      );
    })
    .then(function () {
      let trace = [];
      let held = asyncResource("update", trace);
      let error = { label: "update error" };
      return rejected(
        updateAbruptLoop(trace, held, error),
        error,
        "update abrupt loop",
        function () {
          sameTrace(
            trace,
            [
              "update:get-async",
              "update:body",
              "update:abrupt",
              "update:dispose",
            ],
            "update abrupt lifecycle"
          );
        }
      );
    })
    .then(function () {
      let trace = [];
      let invalid = {};
      invalid[Symbol.asyncDispose] = 1;
      return laterAcquisitionFailure(trace, invalid).then(
        function () {
          throw "later acquisition fulfilled";
        },
        function (error) {
          same(error.name, "TypeError", "later acquisition error");
          sameTrace(
            trace,
            ["acquire:later-tdz:ReferenceError", "acquire:first-dispose"],
            "later acquisition lifecycle"
          );
        }
      );
    })
    .then(function () {
      let trace = [];
      let firstError = { label: "first dispose" };
      let secondError = { label: "second dispose" };
      let bodyError = { label: "body" };
      let first = asyncResource("suppressed:first", trace, firstError);
      let second = asyncResource("suppressed:second", trace, secondError);
      return suppressedLoop(trace, first, second, bodyError).then(
        function () {
          throw "suppressed loop fulfilled";
        },
        function (error) {
          same(error.name, "SuppressedError", "outer suppression brand");
          same(error.error, firstError, "outer suppression error");
          same(error.suppressed.name, "SuppressedError", "inner suppression brand");
          same(error.suppressed.error, secondError, "inner suppression error");
          same(error.suppressed.suppressed, bodyError, "body suppression identity");
          sameTrace(
            trace,
            [
              "suppressed:first:get-async",
              "suppressed:second:get-async",
              "suppressed:body",
              "suppressed:second:dispose",
              "suppressed:first:dispose",
            ],
            "suppressed lifecycle"
          );
          same(first.count(), 1, "suppressed first exactly once");
          same(second.count(), 1, "suppressed second exactly once");
        }
      );
    })
    .then(function () {
      let trace = [];
      let held = asyncResource("capture", trace);
      return fulfilled(captureLoop(trace, held), "capture loop", function (value) {
        same(value, held.value, "captured binding value");
        sameTrace(
          trace,
          [
            "capture:get-async",
            "capture:body:capture:0",
            "capture:dispose",
            "capture:after:outer:capture:1",
          ],
          "capture environment lifecycle"
        );
        same(held.count(), 1, "capture exactly once");
      });
    });
}

main().then(
  function () {
    print("await-using-classic-for:true");
  },
  function (error) {
    print("await-using-classic-for:FAILED:" + error);
    throw error;
  }
);
