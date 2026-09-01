let values = [1, 2, 3, 4];
let forward = values.reduce(function (accumulator, value, index, receiver) {
  if (receiver !== values) throw "reduce receiver";
  return accumulator + value + index;
});
let reverse = values.reduceRight(function (accumulator, value, index) {
  return accumulator * 10 + value + index;
});
let typedValues = new Uint8Array([1, 2, 3, 4]);
let typedForward = typedValues.reduce(function (accumulator, value) {
  return accumulator * 10 + value;
});
let typedReverse = typedValues.reduceRight(function (accumulator, value) {
  return accumulator * 10 + value;
});

forward === 16 &&
  reverse === 4531 &&
  typedForward === 1234 &&
  typedReverse === 4321;
