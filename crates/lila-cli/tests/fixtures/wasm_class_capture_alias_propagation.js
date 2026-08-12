function methodCapture() {
  let x = "outer";
  {
    let x = 2;
    class C {
      m() { return x + 3; }
    }
    return new C().m() === 5;
  }
}

function constructorCapture() {
  let x = "outer";
  {
    let x = 2;
    class C {
      constructor() { this.value = x + 3; }
    }
    return new C().value === 5;
  }
}

function methodRootCapture() {
  let x = "outer";
  {
    let x = 2;
    class C {
      m() {
        function inner() { return x + 3; }
        return inner();
      }
    }
    return new C().m() === 5;
  }
}

function constructorRootCapture() {
  let x = "outer";
  {
    let x = 2;
    class C {
      constructor() {
        function inner() { return x + 3; }
        this.value = inner();
      }
    }
    return new C().value === 5;
  }
}

methodCapture()
  && constructorCapture()
  && methodRootCapture()
  && constructorRootCapture();
