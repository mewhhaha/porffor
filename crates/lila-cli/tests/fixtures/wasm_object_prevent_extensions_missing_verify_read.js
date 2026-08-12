var err = new Error();
Object.preventExtensions(err);
err.exName = "unlikelyValue";

function isWritable(obj, name, verifyProp, value) {
  var newValue = value || "unlikelyValue";
  var hadValue = Object.prototype.hasOwnProperty.call(obj, name);
  var oldValue = obj[name];
  var writeSucceeded;

  if (arguments.length < 4 && newValue === oldValue) {
    newValue = newValue + "2";
  }

  try {
    obj[name] = newValue;
  } catch (e) {
  }

  writeSucceeded = obj[verifyProp || name] === newValue;

  if (writeSucceeded) {
    if (hadValue) {
      obj[name] = oldValue;
    } else {
      delete obj[name];
    }
  }

  return writeSucceeded;
}

err.nocheck === undefined
  && err.hasOwnProperty("exName") === false
  && isWritable(err, "exName", "nocheck") === false
  && err.hasOwnProperty("exName") === false;
