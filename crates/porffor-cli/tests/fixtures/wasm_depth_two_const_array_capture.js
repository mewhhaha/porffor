function outer() {
  const value = 0;

  function middle() {
    function inner() {
      let threwTypeError = false;
      try {
        [value] = [1];
      } catch (error) {
        threwTypeError = error.name === "TypeError";
      }
      return threwTypeError && value === 0;
    }

    return inner;
  }

  return middle;
}

outer()()();
