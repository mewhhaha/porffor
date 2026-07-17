function setDynamic(object, key, value) {
  object[key] = value;
  return object[key];
}

let plainObject = {};
if (setDynamic(plainObject, "plain", "value") !== "value") {
  throw "dynamic plain object result";
}
if (plainObject.plain !== "value") throw "dynamic plain object write";

let array = [1, 2];
if (setDynamic(array, "length", 4294967295) !== 4294967295) {
  throw "dynamic large length result";
}
if (array.length !== 4294967295) throw "dynamic large length";
if (array[0] !== 1 || array[1] !== 2) throw "dynamic length retained elements";
if (array[4294967294] !== undefined) throw "dynamic length high hole";
if (setDynamic(array, "length", 1) !== 1) throw "dynamic length shrink result";
if (array.length !== 1 || array[0] !== 1 || array[1] !== undefined) {
  throw "dynamic length shrink";
}

true;
