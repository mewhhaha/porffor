let trace = "";

function ignoreRejection() {}

function abruptExpression() {
  trace += "e";
  throw 1;
}

function abruptAsyncIteratorGetter() {
  trace += "e";
  return {
    get [Symbol.iterator]() {
      trace += "s";
      return function () {
        return {};
      };
    },
    get [Symbol.asyncIterator]() {
      trace += "a";
      throw 2;
    },
  };
}

function synchronousFallback() {
  trace += "e";
  return {
    get [Symbol.asyncIterator]() {
      trace += "a";
      return null;
    },
    get [Symbol.iterator]() {
      trace += "s";
      return function () {
        trace += "c";
        return {
          get next() {
            trace += "n";
            return 0;
          },
        };
      };
    },
  };
}

function traceMatches(expected) {
  let matches = trace === expected;
  trace = "";
  return matches;
}

function preservesAcquisitionOrder(delegate) {
  delegate(abruptExpression).next().catch(ignoreRejection);
  if (!traceMatches("e")) return false;

  delegate(abruptAsyncIteratorGetter).next().catch(ignoreRejection);
  if (!traceMatches("ea")) return false;

  delegate(synchronousFallback).next().catch(ignoreRejection);
  return traceMatches("eascn");
}

class PublicInstanceDelegate {
  async *delegate(source) {
    yield* source();
  }
}

class PublicStaticDelegate {
  static async *delegate(source) {
    yield* source();
  }
}

class PrivateInstanceDelegate {
  async *#delegate(source) {
    yield* source();
  }

  get delegate() {
    return this.#delegate;
  }
}

class PrivateStaticDelegate {
  static async *#delegate(source) {
    yield* source();
  }

  static get delegate() {
    return this.#delegate;
  }
}

let objectDelegate = {
  async *delegate(source) {
    yield* source();
  },
}.delegate;

let preservedWrapperCount = 0;
let publicInstanceDelegate = PublicInstanceDelegate.prototype.delegate;
if (preservesAcquisitionOrder(publicInstanceDelegate)) preservedWrapperCount += 1;
if (preservesAcquisitionOrder(PublicStaticDelegate.delegate)) preservedWrapperCount += 1;

let privateInstanceDelegate = new PrivateInstanceDelegate().delegate;
if (preservesAcquisitionOrder(privateInstanceDelegate)) preservedWrapperCount += 1;
if (preservesAcquisitionOrder(PrivateStaticDelegate.delegate)) preservedWrapperCount += 1;
if (preservesAcquisitionOrder(objectDelegate)) preservedWrapperCount += 1;

preservedWrapperCount;
