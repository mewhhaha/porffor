var obj = {};
obj.srcArr = [1, 2, 3, 4, 5];

function callbackfn(val, idx, obj) {
  delete obj.srcArr;
  if (val > 0) {
    return 1;
  }
  return 0;
}

var resArr = obj.srcArr.map(callbackfn);
resArr.toString();
