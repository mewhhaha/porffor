var source = [17, NaN, "tail"];
var concatCopy = [].concat(source);
var spreadCopy = [...source];
var shrunk = [17, NaN, "tail"];
shrunk.length = 1;
var shrunkCopy = [].concat(shrunk);
var shrunkValue = shrunkCopy[1];
var notSpread = [17, NaN];
notSpread[Symbol.isConcatSpreadable] = false;
var notSpreadCopy = [].concat(notSpread);
var notSpreadValue = notSpreadCopy[0];
var flattened = [[17, NaN], ["tail"]].flat();
var flattenedValue = flattened[1];
var flatMapped = [1, , 2].flatMap(function (value) {
  return [value];
});
var flatMappedHoleValue = flatMapped[2];

Number.isNaN(source[1])
  && Number.isNaN(concatCopy[1])
  && typeof concatCopy[1] === "number"
  && concatCopy[0] === 17
  && concatCopy[2] === "tail"
  && Number.isNaN(spreadCopy[1])
  && typeof spreadCopy[1] === "number"
  && spreadCopy[0] === 17
  && spreadCopy[2] === "tail"
  && shrunkCopy.length === 1
  && typeof shrunkValue === "undefined"
  && notSpreadCopy.length === 1
  && notSpreadValue === notSpread
  && Number.isNaN(flattenedValue)
  && typeof flattenedValue === "number"
  && flattened[0] === 17
  && flattened[2] === "tail"
  && flatMapped.length === 2
  && typeof flatMappedHoleValue === "undefined";
