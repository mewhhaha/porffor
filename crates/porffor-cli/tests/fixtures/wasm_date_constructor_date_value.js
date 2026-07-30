const dateValue = 1438560000000;
const poisonedDate = new Date(dateValue);
Object.defineProperty(poisonedDate, Symbol.toPrimitive, {
  get() {
    throw "Date @@toPrimitive getter";
  }
});
poisonedDate.valueOf = function() {
  throw "Date valueOf";
};
poisonedDate.toString = function() {
  throw "Date toString";
};

if (new Date(poisonedDate).getTime() !== dateValue) {
  throw "Date value was not copied";
}
if (new Date("1970").toISOString() !== "1970-01-01T00:00:00.000Z") {
  throw "year-only Date string";
}
if (new Date("1970-02").toISOString() !== "1970-02-01T00:00:00.000Z") {
  throw "year-month Date string";
}

const expectedError = {};
const poisonedObject = {};
Object.defineProperty(poisonedObject, Symbol.toPrimitive, {
  get() {
    throw expectedError;
  }
});

if (typeof Date(poisonedObject) !== "string") {
  throw "Date call observed an ignored argument";
}

let observedError;
try {
  new Date(poisonedObject);
} catch (error) {
  observedError = error;
}
if (observedError !== expectedError) {
  throw "ordinary object did not retrieve @@toPrimitive";
}

262;
