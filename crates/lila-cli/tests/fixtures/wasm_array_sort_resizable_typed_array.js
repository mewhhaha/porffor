function compare(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

function sortAfterGrowth(lengthTracking) {
  var buffer = new ArrayBuffer(4, { maxByteLength: 8 });
  var array = lengthTracking
    ? new Uint8Array(buffer, 0)
    : new Uint8Array(buffer, 0, 4);
  var full = new Uint8Array(buffer, 0);
  for (var i = 0; i < full.length; i = i + 1) full[i] = 10 - i;

  Array.prototype.sort.call(array, function(left, right) {
    buffer.resize(6);
    return compare(left, right);
  });

  return buffer.byteLength === 6 && full.length === 6 &&
    full[0] === 7 && full[1] === 8 && full[2] === 9 &&
    full[3] === 10 && full[4] === 0 && full[5] === 0;
}

sortAfterGrowth(false) && sortAfterGrowth(true);
