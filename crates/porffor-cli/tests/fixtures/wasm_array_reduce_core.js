let values = [1, 2, 3, 4];
let forward = values.reduce(function (accumulator, value, index, receiver) {
  if (receiver !== values) throw "reduce receiver";
  return accumulator + value + index;
});
let reverse = values.reduceRight(function (accumulator, value, index) {
  return accumulator * 10 + value + index;
});

forward === 16 && reverse === 4531;
