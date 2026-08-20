function readArgument(propertyKey) {
  return arguments[propertyKey];
}

readArgument("0") === "0" && readArgument("length") === 1;
