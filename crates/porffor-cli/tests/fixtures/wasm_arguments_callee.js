function strictSetCallee(argumentsObject, value) {
  "use strict";
  argumentsObject.callee = value;
  return argumentsObject.callee === value;
}

function mappedCallee(value) {
  var argumentsObject = arguments;
  var initialDescriptor = Object.getOwnPropertyDescriptor(argumentsObject, "callee");
  var identityIsExact = argumentsObject.callee === mappedCallee;
  var initialDescriptorIsCorrect = initialDescriptor.value === mappedCallee
    && initialDescriptor.writable === true
    && initialDescriptor.enumerable === false
    && initialDescriptor.configurable === true;

  var assignmentWorked = strictSetCallee(argumentsObject, value);
  var deletionWorked = delete argumentsObject.callee
    && argumentsObject.hasOwnProperty("callee") === false;

  Object.defineProperty(argumentsObject, "callee", {
    value: 41,
    writable: false,
    enumerable: true,
    configurable: false
  });
  var redefinedDescriptor = Object.getOwnPropertyDescriptor(argumentsObject, "callee");
  var redefineWorked = redefinedDescriptor.value === 41
    && redefinedDescriptor.writable === false
    && redefinedDescriptor.enumerable === true
    && redefinedDescriptor.configurable === false;
  var objectKeysIncludeCallee = Object.keys(argumentsObject).indexOf("callee") !== -1;
  var propertyNamesIncludeCallee = Object.getOwnPropertyNames(argumentsObject).indexOf("callee") !== -1;
  var ownKeysIncludeCallee = Reflect.ownKeys(argumentsObject).indexOf("callee") !== -1;
  var inIncludesCallee = "callee" in argumentsObject;
  var forInIncludesCallee = false;
  for (var key in argumentsObject) {
    if (key === "callee") forInIncludesCallee = true;
  }
  var rejectedRedefinition = false;
  try {
    Object.defineProperty(argumentsObject, "callee", { value: 42 });
  } catch (error) {
    rejectedRedefinition = error instanceof TypeError;
  }

  return identityIsExact
    && initialDescriptorIsCorrect
    && assignmentWorked
    && deletionWorked
    && redefineWorked
    && objectKeysIncludeCallee
    && propertyNamesIncludeCallee
    && ownKeysIncludeCallee
    && inIncludesCallee
    && forInIncludesCallee
    && rejectedRedefinition;
}

function strictCalleeDescriptor() {
  "use strict";
  return Object.getOwnPropertyDescriptor(arguments, "callee");
}

var strictDescriptor = strictCalleeDescriptor();
var secondStrictDescriptor = strictCalleeDescriptor();
var poisonDescriptorIsCorrect = strictDescriptor.get === strictDescriptor.set
  && strictDescriptor.get === secondStrictDescriptor.get
  && strictDescriptor.enumerable === false
  && strictDescriptor.configurable === false;
var strictGetThrows = false;
try {
  (function () {
    "use strict";
    return arguments.callee;
  })();
} catch (error) {
  strictGetThrows = error instanceof TypeError;
}
var strictSetThrows = false;
try {
  (function () {
    "use strict";
    arguments.callee = 1;
  })();
} catch (error) {
  strictSetThrows = error instanceof TypeError;
}

function aliasedInvocation() {
  return arguments.callee;
}
var alias = aliasedInvocation;
var aliasIdentityIsExact = alias() === alias;

function abruptAccessor() {
  var argumentsObject = arguments;
  var marker = {};
  Object.defineProperty(argumentsObject, "callee", {
    configurable: true,
    get: function () {
      throw marker;
    }
  });
  try {
    argumentsObject.callee;
  } catch (error) {
    return error === marker;
  }
  return false;
}

mappedCallee(9)
  && poisonDescriptorIsCorrect
  && strictGetThrows
  && strictSetThrows
  && aliasIdentityIsExact
  && abruptAccessor();
