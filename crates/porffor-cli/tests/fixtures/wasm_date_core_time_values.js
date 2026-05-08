if (Date.now() !== 0) throw "Date.now deterministic";
if (new Date(6.54321).valueOf() !== 6) throw "positive TimeClip";
if (new Date(-6.54321).valueOf() !== -6) throw "negative TimeClip";
if (new Date(-0).valueOf() !== 0) throw "negative zero TimeClip";
if (1 / new Date(-0).valueOf() !== Infinity) throw "positive zero TimeClip";
if (new Date(Infinity).valueOf() === new Date(Infinity).valueOf()) throw "Infinity TimeClip";
if (new Date(-Infinity).valueOf() === new Date(-Infinity).valueOf()) throw "-Infinity TimeClip";
if (new Date(2016, 0, 1, 0, 0, 0, -1).getFullYear() !== 2015) throw "ms underflow";
if (new Date(2016, 11, 31, 23, 59, 59, 1000).getFullYear() !== 2017) throw "ms overflow";
if (new Date(0).getTimezoneOffset() !== 0) throw "timezone offset";
if (new Date(NaN).getTimezoneOffset() === new Date(NaN).getTimezoneOffset()) throw "invalid timezone offset";
let threw = 0;
try { Date.prototype.getTimezoneOffset.call({}); } catch (e) { if (e.name === "TypeError") threw += 1; }
if (threw !== 1) throw "timezone receiver";
if (Date.prototype.getTimezoneOffset.length !== 0) throw "timezone length";
if (Date.prototype.getTimezoneOffset.name !== "getTimezoneOffset") throw "timezone name";
let constructThrew = 0;
try { new Date.prototype.getTimezoneOffset(); } catch (e) { if (e.name === "TypeError") constructThrew = 1; }
if (constructThrew !== 1) throw "timezone construct";
262;
