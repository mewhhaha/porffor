function throwsTypeError(fn) {
  try {
    fn();
  } catch (err) {
    return err instanceof TypeError;
  }
  return false;
}

var arr = new Array(10);
throwsTypeError(function () {
  arr.forEach();
});
