var seen = [];

function record(value) {
  seen.push(value);
  return value;
}

let [, selected = record("default")] = [
  record("leading"),
  ,
  record("trailing"),
];
if (
  selected !== "default" ||
  seen.length !== 3 ||
  seen[0] !== "leading" ||
  seen[1] !== "trailing" ||
  seen[2] !== "default"
) {
  throw "lexical pattern evaluation order";
}

let [] = [record("empty-pattern")];
if (seen.length !== 4 || seen[3] !== "empty-pattern") {
  throw "empty lexical pattern evaluation";
}

var assigned;
var assignmentResult = ([, assigned] = [
  record("assignment-leading"),
  record("assignment-selected"),
  record("assignment-trailing"),
]);
if (
  assigned !== "assignment-selected" ||
  assignmentResult[1] !== "assignment-selected" ||
  seen.length !== 7 ||
  seen[4] !== "assignment-leading" ||
  seen[5] !== "assignment-selected" ||
  seen[6] !== "assignment-trailing"
) {
  throw "assignment pattern evaluation order";
}

true;
