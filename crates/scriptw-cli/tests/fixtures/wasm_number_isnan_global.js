let actual = Number.isNaN(NaN);
if (actual !== true) throw actual;
if (Number.isNaN("NaN") !== false) throw "string must not coerce";
if (Number.isFinite(Infinity) !== false) throw "infinity must stay non-finite";
262;
