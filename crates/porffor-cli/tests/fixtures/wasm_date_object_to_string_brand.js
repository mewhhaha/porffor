var values = [
  new Date(0),
  new Date(1899, 11, 31, 23, 59, 59),
  new Date(2100, 0, 1, 0, 0, 0, 0),
];

for (var i = 0; i < values.length; i++) {
  if (Object.prototype.toString.call(values[i]) !== "[object Date]") {
    throw "direct brand";
  }
}

var forgedDate = { "$DateValue": 0 };
var inheritedDateValue = Object.create(forgedDate);
for (var k = 0; k < 2; k++) {
  var receiver = k === 0 ? forgedDate : inheritedDateValue;
  if (Object.prototype.toString.call(receiver) !== "[object Object]") {
    throw "forged brand";
  }

  var threw = false;
  try {
    Date.prototype.getTime.call(receiver);
  } catch (error) {
    threw = error instanceof TypeError;
  }
  if (!threw) throw "forged receiver";
}

var taggedDate = new Date(0);
taggedDate[Symbol.toStringTag] = "Clock";
if (Object.prototype.toString.call(taggedDate) !== "[object Clock]") {
  throw "custom tag";
}

Date.prototype.toString = Object.prototype.toString;

for (var j = 0; j < values.length; j++) {
  if (values[j].toString() !== "[object Date]") {
    throw "overridden method";
  }
}

262;
