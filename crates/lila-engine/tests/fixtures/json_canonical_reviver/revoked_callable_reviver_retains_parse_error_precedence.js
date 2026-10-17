function check(text, label) {
  var holder = Proxy.revocable(function () { throw "revoked target ran"; }, {});
  var input = {[Symbol.toPrimitive]: function () {
    print("coerce:" + label);
    holder.revoke();
    return text;
  }};
  try { JSON.parse(input, holder.proxy); print("unexpected success"); }
  catch (error) { print(label + ":" + (error instanceof SyntaxError) + ":" + (error instanceof TypeError)); }
}
check("[1]", "valid");
check("[1,]", "invalid");
