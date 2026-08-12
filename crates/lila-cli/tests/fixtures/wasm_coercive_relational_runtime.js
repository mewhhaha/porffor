let order = "";
let left = {
  valueOf() {
    order += "L";
    return 2;
  }
};
let right = {
  valueOf() {
    order += "R";
    return 3;
  }
};

if (!(left < right)) throw "object relational result";
if (order !== "LR") throw "object relational order";
if (!(right >= left)) throw "object relational greater equal";

let actual = ["a", "b"];
let expected = ["a", "b"];
if (!(actual.length <= expected.length)) throw "array length relational";
if (!([1, 2] < [1, 3])) throw "array string relational";
if (!("2" < 3)) throw "string number relational";

123;
