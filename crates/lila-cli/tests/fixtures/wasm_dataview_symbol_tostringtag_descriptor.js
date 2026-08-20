let desc = Object.getOwnPropertyDescriptor(DataView.prototype, Symbol.toStringTag);

(DataView.prototype[Symbol.toStringTag] == "DataView") +
  (desc.value == "DataView") +
  (desc.writable == false) +
  (desc.enumerable == false) +
  (desc.configurable == true);
