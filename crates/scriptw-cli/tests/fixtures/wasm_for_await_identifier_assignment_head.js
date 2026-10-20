let async = 0;
const immutable = 3;
let shadowed = 11;
var declared = 0;

async function assignOuter() {
  for await (async of [1, 7]) {}
}

async function verifyIdentifierHeads() {
  await assignOuter();
  if (async !== 7) throw "bare identifier did not update its outer binding";

  let immutableWriteThrew = false;
  try {
    for await (immutable of [7]) {}
  } catch (error) {
    immutableWriteThrew = error instanceof TypeError;
  }
  if (!immutableWriteThrew) throw "bare const target did not throw TypeError";
  if (immutable !== 3) throw "bare const target was mutated";

  for await (let shadowed of [7]) {
    if (shadowed !== 7) throw "let declaration received the wrong value";
  }
  if (shadowed !== 11) throw "let declaration mutated its outer namesake";

  for await (var declared of [7]) {}
  if (declared !== 7) throw "var declaration did not update its declared binding";
}

verifyIdentifierHeads();
262;
