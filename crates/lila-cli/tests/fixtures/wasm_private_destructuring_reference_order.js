class ReturnReceiver {
  constructor(receiver) {
    return receiver;
  }
}

class PrivateField extends ReturnReceiver {
  #value;

  installDuringSourceRead() {
    const install = () => new PrivateField(this);
    const source = {
      get value() {
        install();
        return 42;
      },
    };

    ({ value: this.#value } = source);
    return this.#value;
  }
}

const result = PrivateField.prototype.installDuringSourceRead.call({});
if (result !== 42) throw "private destructuring assignment result";

true;
