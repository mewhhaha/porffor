let $262 = { createRealm: __lilaCreateRealm };
let other = $262.createRealm().global;

if (typeof other.RegExp.prototype.exec !== "function") {
  throw "created realm RegExp.prototype.exec is not callable";
}
if (other.RegExp.prototype.exec === RegExp.prototype.exec) {
  throw "created realm RegExp.prototype.exec identity";
}

let mainExec = RegExp.prototype.exec;
let foreignExec = other.RegExp.prototype.exec;
RegExp.prototype.exec = foreignExec;

let match = /a/[Symbol.match]("ba");
let search = /a/[Symbol.search]("ba");
RegExp.prototype.exec = mainExec;

if (!Array.isArray(match) || match[0] !== "a" || match.index !== 1) {
  throw "foreign realm exec match fallback";
}
if (search !== 1) {
  throw "foreign realm exec search fallback";
}

let mainExecDescriptor = Object.getOwnPropertyDescriptor(RegExp.prototype, "exec");
let accessorReads = 0;
let accessorCalls = 0;
Object.defineProperty(RegExp.prototype, "exec", {
  configurable: true,
  get() {
    accessorReads++;
    return function () {
      accessorCalls++;
      return { index: 8 };
    };
  },
});
let accessorSearch = /a/[Symbol.search]("ba");
Object.defineProperty(RegExp.prototype, "exec", mainExecDescriptor);

if (accessorSearch !== 8 || accessorReads !== 1 || accessorCalls !== 1) {
  throw "accessor exec search fallback";
}

true;
