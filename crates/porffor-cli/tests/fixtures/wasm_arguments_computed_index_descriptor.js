function inspectArgument(propertyKey) {
  const descriptor = Object.getOwnPropertyDescriptor(arguments, propertyKey);
  const lengthDescriptor = Object.getOwnPropertyDescriptor(arguments, "length");
  return arguments[propertyKey] === propertyKey
    && descriptor.value === propertyKey
    && descriptor.writable === true
    && descriptor.enumerable === true
    && descriptor.configurable === true
    && lengthDescriptor.value === 1
    && lengthDescriptor.writable === true
    && lengthDescriptor.enumerable === false
    && lengthDescriptor.configurable === true;
}

inspectArgument("0");
