var ok = true;

var calls = "";
var proxy = new Proxy({}, {
  get: function(_, key) {
    calls = calls + key + ",";
    if (key === "length") return 4;
    return key * 10;
  }
});

var proxyMiss = [].includes.call(proxy, 42);
var proxyMissCalls = calls === "length,0,1,2,3,";

calls = "";
var proxyHit = [].includes.call(proxy, 10);
var proxyHitCalls = calls === "length,0,1,";

var spoofReads = "";
var spoof = {
  $TypedArrayByteLength: 2,
  get length() {
    spoofReads = spoofReads + "length,";
    return 2;
  },
  get 0() {
    spoofReads = spoofReads + "0,";
    return "miss";
  },
  get 1() {
    spoofReads = spoofReads + "1,";
    return "hit";
  }
};
var spoofHit = [].includes.call(spoof, "hit");
var spoofReadCalls = spoofReads === "length,0,1,";

var floats = new Float32Array(1);
floats[0] = NaN;
var typedNaNHit = [].includes.call(floats, NaN);

var typedLengthReads = 0;
var typedWithLengthGetter = new Uint8Array(2);
typedWithLengthGetter[0] = 1;
typedWithLengthGetter[1] = 2;
Object.defineProperty(typedWithLengthGetter, "length", {
  configurable: true,
  get: function () {
    typedLengthReads += 1;
    return 1;
  }
});
var typedLengthGetterObserved =
  ![].includes.call(typedWithLengthGetter, 2) && typedLengthReads === 1;

var typedWithDataLength = new Uint8Array(2);
typedWithDataLength[0] = 1;
typedWithDataLength[1] = 2;
Object.defineProperty(typedWithDataLength, "length", {
  configurable: true,
  value: 1
});
var typedDataLengthObserved = ![].includes.call(typedWithDataLength, 2);

ok && !proxyMiss && proxyMissCalls && proxyHit && proxyHitCalls &&
  spoofHit && spoofReadCalls && typedNaNHit && typedLengthGetterObserved &&
  typedDataLengthObserved;
