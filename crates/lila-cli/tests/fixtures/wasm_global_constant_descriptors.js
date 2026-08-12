Infinity = true;
NaN = true;
undefined = true;

let infinityDesc = Object.getOwnPropertyDescriptor(this, "Infinity");
let nanDesc = Object.getOwnPropertyDescriptor(this, "NaN");
let undefinedDesc = Object.getOwnPropertyDescriptor(this, "undefined");

typeof Infinity === "number"
  && typeof NaN === "number"
  && typeof undefined === "undefined"
  && infinityDesc.writable === false
  && infinityDesc.enumerable === false
  && infinityDesc.configurable === false
  && nanDesc.writable === false
  && nanDesc.enumerable === false
  && nanDesc.configurable === false
  && undefinedDesc.writable === false
  && undefinedDesc.enumerable === false
  && undefinedDesc.configurable === false;
