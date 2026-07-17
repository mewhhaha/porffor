function writeDynamic(target, key, value) {
  target[key] = value;
  return target[key];
}

let array = [];
if (writeDynamic(array, "0", "first") !== "first" || array.length !== 1) {
  throw "dynamic dense index write";
}
if (writeDynamic(array, "1000000", "sparse") !== "sparse") {
  throw "dynamic sparse index write";
}
if (array[1000000] !== "sparse" || array.length !== 1000001) {
  throw "dynamic sparse index result";
}
if (writeDynamic(array, "4294967294", "last") !== "last") {
  throw "dynamic maximum array index write";
}
if (array[4294967294] !== "last" || array.length !== 4294967295) {
  throw "dynamic maximum array index result";
}

writeDynamic(array, "01", "named");
writeDynamic(array, "4294967295", "named maximum");
if (array["01"] !== "named" || array[4294967295] !== "named maximum") {
  throw "noncanonical dynamic named writes";
}
if (array.length !== 4294967295) throw "non-index dynamic keys changed length";

let object = {};
if (writeDynamic(object, "0", "object index") !== "object index") {
  throw "dynamic object index result";
}
if (object[0] !== "object index") throw "dynamic object index write";

true;
