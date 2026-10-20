(function (global) {
  global.dynamicGlobalFunction = function () {
    return 42;
  };
})(this);

if (dynamicGlobalFunction() !== 42) {
  throw "dynamic global function";
}

delete globalThis.dynamicGlobalFunction;
var missingThrew = false;
try {
  dynamicGlobalFunction();
} catch (error) {
  missingThrew = error instanceof ReferenceError;
}
if (!missingThrew) {
  throw "missing dynamic global function";
}

print("dynamic-global-identifier:true");
