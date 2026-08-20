let view = new DataView(new ArrayBuffer(8));
let boxed = Object(Symbol("boxed"));
let score = 0;

function throwsTypeError(fn) {
  try {
    fn();
  } catch (error) {
    return error instanceof TypeError;
  }
  return false;
}

if (throwsTypeError(function () { view.getBigInt64(boxed); })) score += 1;
if (throwsTypeError(function () { view.getBigUint64(boxed); })) score += 1;

score === 2 ? 2 : score;
