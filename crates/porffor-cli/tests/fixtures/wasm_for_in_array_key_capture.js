let values = new Array();
values[0] = "zero";
values[5] = "five";
values[31] = "last";

let visited = 0;
let ok = true;

for (var index in values) {
  (function () {
    let expected = index === "0" ? "zero" : index === "5" ? "five" : "last";
    if (values[index] !== expected) ok = false;
    visited = visited + 1;
  })();
}

ok && visited === 3;
