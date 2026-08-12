function Test262Error(message) {
}

function check(value, message) {
  if (!value) {
    throw new Test262Error(message);
  }
}

var __instance = new Object(42);

__instance.charAt = String.prototype.charAt;

check(__instance.charAt(false) + __instance.charAt(true) === "42", "#1 object charAt");

__instance = new Boolean;

__instance.charAt = String.prototype.charAt;

check(__instance.charAt(false) + __instance.charAt(true) + __instance.charAt(true + 1) === "fal", "#1 boolean charAt");

true;
