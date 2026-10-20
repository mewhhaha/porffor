var observations = [];
var nextCall = 0;
var spread = {};

spread[Symbol.iterator] = function () {
  observations.push("iterator");
  return {
    next: function () {
      observations.push("next " + nextCall);
      if (nextCall === 0) {
        nextCall += 1;
        return { value: "spread 0", done: false };
      }
      if (nextCall === 1) {
        nextCall += 1;
        return { value: "spread 1", done: false };
      }
      nextCall += 1;
      return { value: undefined, done: true };
    },
  };
};

function* buildArray() {
  yield [
    (observations.push("prefix"), "prefix"),
    ...(yield "suspended"),
    (observations.push("suffix"), "suffix"),
  ];
}

var iterator = buildArray();
var suspended = iterator.next();
var prefixWasCommittedBeforeSuspension =
  suspended.done === false &&
  suspended.value === "suspended" &&
  observations.length === 1 &&
  observations[0] === "prefix";

var yielded = iterator.next(spread);
var accumulated = yielded.value;
var completed = iterator.next();

prefixWasCommittedBeforeSuspension &&
  yielded.done === false &&
  Array.isArray(accumulated) &&
  accumulated.length === 4 &&
  accumulated[0] === "prefix" &&
  accumulated[1] === "spread 0" &&
  accumulated[2] === "spread 1" &&
  accumulated[3] === "suffix" &&
  observations.length === 6 &&
  observations[0] === "prefix" &&
  observations[1] === "iterator" &&
  observations[2] === "next 0" &&
  observations[3] === "next 1" &&
  observations[4] === "next 2" &&
  observations[5] === "suffix" &&
  completed.done === true &&
  completed.value === undefined;
