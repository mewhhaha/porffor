class ReturnReceiver {
  constructor(receiver) {
    return receiver;
  }
}

class PrivateMethod extends ReturnReceiver {
  #method() {}
}

class PrivateGetter extends ReturnReceiver {
  get #value() {}
}

class PrivateSetter extends ReturnReceiver {
  set #value(value) {}
}

class PrivateAccessorPair extends ReturnReceiver {
  get #value() {}
  set #value(value) {}
}

class PrivateField extends ReturnReceiver {
  #value;
}

function assertSecondInstallationThrows(Constructor, message) {
  const receiver = {};
  new Constructor(receiver);

  try {
    new Constructor(receiver);
  } catch (error) {
    if (error.name === "TypeError") return;
  }

  throw message;
}

assertSecondInstallationThrows(PrivateMethod, "private method installed twice");
assertSecondInstallationThrows(PrivateGetter, "private getter installed twice");
assertSecondInstallationThrows(PrivateSetter, "private setter installed twice");
assertSecondInstallationThrows(PrivateAccessorPair, "private accessor pair installed twice");
assertSecondInstallationThrows(PrivateField, "private field installed twice");

true;
