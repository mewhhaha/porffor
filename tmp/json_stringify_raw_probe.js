if (JSON.stringify(JSON.rawJSON(1)) !== "1") throw "top raw";
if (JSON.stringify(JSON.rawJSON('"foo"')) !== '"foo"') throw "top raw string";
if (JSON.stringify({ 42: JSON.rawJSON(37) }) !== '{"42":37}') throw "object raw";
if (JSON.stringify({ x: { x: JSON.rawJSON(1), y: JSON.rawJSON(2) } }) !== '{"x":{"x":1,"y":2}}') throw "nested raw";
if (JSON.stringify([JSON.rawJSON(1), JSON.rawJSON(1.1)]) !== "[1,1.1]") throw "array raw";
