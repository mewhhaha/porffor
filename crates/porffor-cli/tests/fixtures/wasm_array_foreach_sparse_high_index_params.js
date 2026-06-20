var bPar = true;
var bCalled = false;
var callCount = 0;
var objValue = new Object();

function callbackfn(val, idx, obj) {
  bCalled = true;
  callCount++;
  if (obj[idx] !== val) {
    bPar = false;
  }
}

var arr = [0, 1, true, null, objValue, "five"];
arr[999999] = -6.6;
arr.forEach(callbackfn);

bCalled === true && bPar === true && callCount === 7;
