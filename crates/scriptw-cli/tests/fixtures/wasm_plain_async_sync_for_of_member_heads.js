function same(actual, expected, label) {
  if (actual !== expected) throw label + ": " + actual;
}

async function rejectionOf(promise, label) {
  try {
    await promise;
  } catch (error) {
    return error;
  }
  throw label + " fulfilled";
}

const staticTarget = { value: 0 };
let staticTrace = "";

async function consumeStaticMember() {
  for (staticTarget.value of [3, 5]) {
    staticTrace += "body:" + staticTarget.value + ">";
    await 0;
    staticTrace += "resume:" + staticTarget.value + ">";
  }
}

let memberTrace = "";
let memberValue = 0;
const firstMemberTarget = {};
const secondMemberTarget = {};
Object.defineProperty(firstMemberTarget, "first", {
  set: function (value) {
    memberValue = value;
    memberTrace += "set:first:" + value + ">";
  },
});
Object.defineProperty(secondMemberTarget, "second", {
  set: function (value) {
    memberValue = value;
    memberTrace += "set:second:" + value + ">";
  },
});
let activeMemberTarget = firstMemberTarget;
let activeMemberKey = "first";
let memberIteratorIndex = 0;

function memberBase() {
  memberTrace +=
    "base:" + (activeMemberTarget === firstMemberTarget ? "first>" : "second>");
  return activeMemberTarget;
}

function memberKey() {
  memberTrace += "key:" + activeMemberKey + ">";
  return activeMemberKey;
}

const memberIterable = {
  [Symbol.iterator]: function () {
    memberTrace += "iterator>";
    return {
      next: function () {
        memberTrace += "next:" + memberIteratorIndex + ">";
        if (memberIteratorIndex === 0) {
          memberIteratorIndex++;
          return { value: 4, done: false };
        }
        if (memberIteratorIndex === 1) {
          memberIteratorIndex++;
          return { value: 8, done: false };
        }
        return { value: undefined, done: true };
      },
    };
  },
};

async function consumeComputedMember() {
  for (memberBase()[memberKey()] of memberIterable) {
    memberTrace += "body:" + memberValue + ">";
    if (memberValue === 4) {
      activeMemberTarget = secondMemberTarget;
      activeMemberKey = "second";
    }
    await 0;
    memberTrace += "resume:" + memberValue + ">";
  }
}

const setterError = { label: "setter" };
const setterCloseError = { label: "setter close" };
const throwingTarget = {};
Object.defineProperty(throwingTarget, "value", {
  set: function () {
    throw setterError;
  },
});
let setterCloseCalls = 0;
let setterBodyCalls = 0;
const closingIterable = {
  [Symbol.iterator]: function () {
    return {
      next: function () {
        return { value: 11, done: false };
      },
      return: function () {
        setterCloseCalls++;
        throw setterCloseError;
      },
    };
  },
};

async function rejectThrowingSetter() {
  for (throwingTarget.value of closingIterable) {
    setterBodyCalls++;
    await 0;
  }
}

const privateCloseError = { label: "private close" };
let privateCloseCalls = 0;
let privateBodyCalls = 0;
const privateClosingIterable = {
  [Symbol.iterator]: function () {
    return {
      next: function () {
        return { value: 13, done: false };
      },
      return: function () {
        privateCloseCalls++;
        throw privateCloseError;
      },
    };
  },
};

class PrivateMemberTarget {
  #value = 0;

  async consume() {
    let trace = "";
    for (this.#value of [7, 9]) {
      trace += "body:" + this.#value + ">";
      await 0;
      trace += "resume:" + this.#value + ">";
    }
    return this.#value + ":" + trace;
  }

  async rejectWrongBrand(wrong) {
    for (wrong.#value of privateClosingIterable) {
      privateBodyCalls++;
      await 0;
    }
  }
}

async function main() {
  await consumeStaticMember();
  same(staticTarget.value, 5, "static member final value");
  same(
    staticTrace,
    "body:3>resume:3>body:5>resume:5>",
    "static member await trace"
  );

  await consumeComputedMember();
  same(
    memberTrace,
    "iterator>next:0>base:first>key:first>set:first:4>body:4>resume:4>" +
      "next:1>base:second>key:second>set:second:8>body:8>resume:8>next:2>",
    "computed member lifecycle"
  );

  same(
    await rejectionOf(rejectThrowingSetter(), "throwing setter loop"),
    setterError,
    "setter rejection identity"
  );
  same(setterCloseCalls, 1, "setter IteratorClose count");
  same(setterBodyCalls, 0, "setter body calls");

  const privateTarget = new PrivateMemberTarget();
  same(
    await privateTarget.consume(),
    "9:body:7>resume:7>body:9>resume:9>",
    "private member await trace"
  );
  const privateError = await rejectionOf(
    privateTarget.rejectWrongBrand({}),
    "private wrong-brand loop"
  );
  same(privateError.name, "TypeError", "private wrong-brand error");
  same(privateError === privateCloseError, false, "private throw precedence");
  same(privateCloseCalls, 1, "private IteratorClose count");
  same(privateBodyCalls, 0, "private body calls");

  print("plain-async-sync-for-of:member-heads=ok");
}

main().then(undefined, function (error) {
  print("plain-async-sync-for-of:member-heads=FAILED:" + error);
});

0;
