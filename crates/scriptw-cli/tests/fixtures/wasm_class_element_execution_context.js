function blockCapture() {
  let nearest = 1;
  {
    let nearest = 2;
    class C {
      publicValue = nearest;
      #privateValue = nearest;
      static snapshot = nearest;
      static { this.readNearest = () => nearest; }
      readPrivate() { return this.#privateValue; }
    }
    nearest = 3;
    let instance = new C();
    return instance.publicValue === 3
      && instance.readPrivate() === 3
      && C.snapshot === 2
      && C.readNearest() === 3;
  }
}

function switchCapture() {
  let nearest = 10;
  switch (0) {
    case 0:
      let nearest = 20;
      class C {
        instanceValue = nearest;
        static snapshot = nearest;
      }
      nearest = 21;
      let instance = new C();
      return instance.instanceValue === 21 && C.snapshot === 20;
  }
}

function catchCapture() {
  let caught = 30;
  try {
    throw 40;
  } catch (caught) {
    class C {
      instanceValue = caught;
      static snapshot = caught;
      static { this.readCaught = () => caught; }
    }
    caught = 41;
    let instance = new C();
    return instance.instanceValue === 41
      && C.snapshot === 40
      && C.readCaught() === 41;
  }
}

class Base {}
Base.prototype.marker = 50;
Base.marker = 60;

class Derived extends Base {
  direct = super.marker;
  arrow = (() => super.marker)();
  static direct = super.marker;
  static arrow = (() => super.marker)();
  static { this.block = (() => super.marker)(); }
}

let derived = new Derived();

blockCapture()
  && switchCapture()
  && catchCapture()
  && derived.direct === 50
  && derived.arrow === 50
  && Derived.direct === 60
  && Derived.arrow === 60
  && Derived.block === 60;
