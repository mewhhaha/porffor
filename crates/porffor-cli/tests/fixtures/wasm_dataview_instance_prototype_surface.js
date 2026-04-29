let view = new DataView(new ArrayBuffer(4));
if (!Object.isExtensible(view)) throw "extensible";
Object.defineProperty(view, "x", { value: 7 });
if (view.x !== 7) throw "define property";
if (view.constructor !== DataView) throw "constructor";
Object.defineProperty(view, "baz", {});
if (!view.hasOwnProperty("baz")) throw "has own baz";
Object.defineProperty(view, "foo", {
  value: "bar",
  writable: true,
  configurable: true,
  enumerable: false
});
function verifyDataDescriptor(obj, name, desc) {
  var originalDesc = Object.getOwnPropertyDescriptor(obj, name);
  if (desc.value !== undefined) {
    if (originalDesc.value !== desc.value) throw "foo descriptor value";
    if (obj[name] !== desc.value) throw "foo value";
  }
  if (desc.writable !== undefined) {
    if (originalDesc.writable !== desc.writable) throw "foo descriptor writable";
  }
  if (desc.configurable !== undefined) {
    if (originalDesc.configurable !== desc.configurable) throw "foo descriptor configurable";
  }
  if (desc.enumerable !== undefined) {
    if (originalDesc.enumerable !== desc.enumerable) throw "foo descriptor enumerable";
  }
}
verifyDataDescriptor(view, "foo", {
  value: "bar",
  writable: true,
  configurable: true,
  enumerable: false
});

let desc = Object.getOwnPropertyDescriptor(DataView, "prototype");
if (desc === undefined) throw "prototype descriptor";

321;
