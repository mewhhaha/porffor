function MyArray() {
  this.length = 4;
  for (let index = 0; index < this.length; index++) {
    Object.defineProperty(this, index, {
      enumerable: true,
      writable: false,
      configurable: true,
      value: 99,
    });
  }
}

Array.fromAsync.call(MyArray, [0, 1, 2]).then(function (result) {
  print(
    "array-from-async-custom-constructor:" +
      (result instanceof MyArray) +
      ":" +
      result.length +
      ":" +
      result[0] +
      ":" +
      result[1] +
      ":" +
      result[2] +
      ":" +
      result[3],
  );
});

0;
