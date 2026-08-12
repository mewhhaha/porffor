function invoke(callback) {
  return callback();
}

for (let binding; ; ) {
  break;
}

let total = 0;
for (let value = 0; value < 2; value++) {
  total += value;
}

if (total !== 1) throw "classic for lexical update";

invoke(function () {
  return 9;
});
