let directArray = [];
let directValue = { marker: 1 };
let directCalls = 0;
let directReceiver = false;
Object.defineProperty(directArray, Symbol.isConcatSpreadable, {
  get() {
    directCalls += 1;
    directReceiver = this === directArray;
    return directValue;
  }
});
let directRead = directArray[Symbol.isConcatSpreadable];

let proxyArray = [];
let proxyValue = { marker: 2 };
let proxyCalls = 0;
let proxyReceiver = false;
let proxyGetter = new Proxy(function() {}, {
  apply(target, receiver, args) {
    proxyCalls += 1;
    proxyReceiver = receiver === proxyArray && args.length === 0;
    return proxyValue;
  }
});
Object.defineProperty(proxyArray, Symbol.isConcatSpreadable, {
  get: proxyGetter
});
let proxyRead = proxyArray[Symbol.isConcatSpreadable];

let inheritedArray = [];
let inheritedValue = { marker: 3 };
let inheritedCalls = 0;
let inheritedReceiver = false;
Object.defineProperty(Array.prototype, Symbol.isConcatSpreadable, {
  configurable: true,
  get() {
    inheritedCalls += 1;
    inheritedReceiver = this === inheritedArray;
    return inheritedValue;
  }
});
let inheritedRead = inheritedArray[Symbol.isConcatSpreadable];

directCalls === 1
  && directReceiver
  && directRead === directValue
  && proxyCalls === 1
  && proxyReceiver
  && proxyRead === proxyValue
  && inheritedCalls === 1
  && inheritedReceiver
  && inheritedRead === inheritedValue;
