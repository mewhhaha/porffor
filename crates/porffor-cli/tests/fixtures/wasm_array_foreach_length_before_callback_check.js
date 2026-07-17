function catchesTypeError(fn) {
  try {
    fn();
  } catch (error) {
    return error.name === "TypeError";
  }
  return false;
}

var mapAccessed = false;
var mapReceiver = {
  0: 11,
  1: 12
};

Object.defineProperty(mapReceiver, "length", {
  get: function() {
    mapAccessed = true;
    return 2;
  },
  configurable: true
});

var mapThrew = catchesTypeError(function() {
  Array.prototype.map.call(mapReceiver, null);
});

var everyAccessed = false;
var everyReceiver = {
  0: 11,
  1: 12
};

Object.defineProperty(everyReceiver, "length", {
  get: function() {
    everyAccessed = true;
    return 2;
  },
  configurable: true
});

var everyThrew = catchesTypeError(function() {
  Array.prototype.every.call(everyReceiver, null);
});

var filterAccessed = false;
var filterReceiver = {
  0: 11,
  1: 12
};

Object.defineProperty(filterReceiver, "length", {
  get: function() {
    filterAccessed = true;
    return 2;
  },
  configurable: true
});

var filterThrew = catchesTypeError(function() {
  Array.prototype.filter.call(filterReceiver, null);
});

var someAccessed = false;
var someReceiver = {
  0: 11,
  1: 12
};

Object.defineProperty(someReceiver, "length", {
  get: function() {
    someAccessed = true;
    return 2;
  },
  configurable: true
});

var someThrew = catchesTypeError(function() {
  Array.prototype.some.call(someReceiver, null);
});

var forEachAccessed = false;
var forEachReceiver = {
  0: 11,
  1: 12
};

Object.defineProperty(forEachReceiver, "length", {
  get: function() {
    forEachAccessed = true;
    return {
      toString: function() {
        forEachAccessed = true;
        return "2";
      }
    };
  },
  configurable: true
});

var forEachThrew = catchesTypeError(function() {
  Array.prototype.forEach.call(forEachReceiver, null);
});

var abrupt = {};
var proxy = new Proxy({}, {
  get: function(_, key) {
    if (key === "length") throw abrupt;
  }
});
var proxyError;
try {
  Array.prototype.some.call(proxy, null);
} catch (error) {
  proxyError = error;
}

var originalSymbolLength = Object.getOwnPropertyDescriptor(Symbol.prototype, "length");
var symbolLengthAccessed = false;
Object.defineProperty(Symbol.prototype, "length", {
  get: function() {
    symbolLengthAccessed = true;
    return 0;
  },
  configurable: true
});
var symbolThrew = catchesTypeError(function() {
  Array.prototype.forEach.call(Symbol("receiver"), null);
});
if (originalSymbolLength === undefined) {
  delete Symbol.prototype.length;
} else {
  Object.defineProperty(Symbol.prototype, "length", originalSymbolLength);
}

var originalBigIntLength = Object.getOwnPropertyDescriptor(BigInt.prototype, "length");
var abruptBigIntLength = {};
Object.defineProperty(BigInt.prototype, "length", {
  get: function() {
    throw abruptBigIntLength;
  },
  configurable: true
});
var bigintError;
try {
  Array.prototype.forEach.call(1n, null);
} catch (error) {
  bigintError = error;
}
if (originalBigIntLength === undefined) {
  delete BigInt.prototype.length;
} else {
  Object.defineProperty(BigInt.prototype, "length", originalBigIntLength);
}

mapThrew && mapAccessed &&
  everyThrew && everyAccessed &&
  filterThrew && filterAccessed &&
  someThrew && someAccessed &&
  forEachThrew && forEachAccessed &&
  proxyError === abrupt &&
  symbolThrew && symbolLengthAccessed &&
  bigintError === abruptBigIntLength;
