function bare() {
  "use strict";
  return this === undefined;
}

var alias = bare;
var callbackThis;

function callback(value, index, receiver) {
  "use strict";
  callbackThis = this;
  return value + index + receiver.length;
}

[1].forEach(callback);

bare() && alias() && callbackThis === undefined;
