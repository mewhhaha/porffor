let methodKey = "dyn";
let readKey = "read";
let writeKey = "write";

let target = {
  marker: 5,
  [methodKey]() { return this.marker + 6; },
  get [readKey]() { return this.marker + 1; },
  set [writeKey](next) { this.marker = next; }
};

let read = Object.getOwnPropertyDescriptor(target, "read").get;
let write = Object.getOwnPropertyDescriptor(target, "write").set;
write.call(target, 9);

target.dyn() === 15 && read.call(target) === 10;
