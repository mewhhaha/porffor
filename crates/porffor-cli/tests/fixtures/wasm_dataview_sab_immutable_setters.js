let sab = new SharedArrayBuffer(8);
let view = new DataView(sab, 1, 4);

if (view.buffer !== sab) throw "sab buffer";
if (view.byteOffset !== 1) throw "sab byteOffset";
if (view.byteLength !== 4) throw "sab byteLength";

view.setUint8(0, 0x7f);
view.setUint8(1, 0xff);
view.setUint8(2, 0xff);
view.setUint8(3, 0xff);
if (view.getInt32(0, false) !== 2147483647) throw "sab getInt32";
if (new DataView(sab).getUint8(1) !== 0x7f) throw "sab write shared storage";

let immutable = new ArrayBuffer(8).transferToImmutable();
let immutableView = new DataView(immutable);
let calls = 0;
let badOffset = {
  valueOf() {
    calls = calls + 1;
    return 0;
  },
};
let badNumber = {
  valueOf() {
    calls = calls + 10;
    return 1;
  },
};
let badBigInt = {
  valueOf() {
    calls = calls + 100;
    return 1n;
  },
};

let threw = false;
try { immutableView.setInt8(badOffset, badNumber); } catch (error) { if (!(error instanceof TypeError)) throw "setInt8 error"; threw = true; }
if (!threw || calls !== 0) throw "setInt8 immutable";
threw = false;
try { immutableView.setUint8(badOffset, badNumber); } catch (error) { if (!(error instanceof TypeError)) throw "setUint8 error"; threw = true; }
if (!threw || calls !== 0) throw "setUint8 immutable";
threw = false;
try { immutableView.setInt16(badOffset, badNumber); } catch (error) { if (!(error instanceof TypeError)) throw "setInt16 error"; threw = true; }
if (!threw || calls !== 0) throw "setInt16 immutable";
threw = false;
try { immutableView.setUint16(badOffset, badNumber); } catch (error) { if (!(error instanceof TypeError)) throw "setUint16 error"; threw = true; }
if (!threw || calls !== 0) throw "setUint16 immutable";
threw = false;
try { immutableView.setInt32(badOffset, badNumber); } catch (error) { if (!(error instanceof TypeError)) throw "setInt32 error"; threw = true; }
if (!threw || calls !== 0) throw "setInt32 immutable";
threw = false;
try { immutableView.setUint32(badOffset, badNumber); } catch (error) { if (!(error instanceof TypeError)) throw "setUint32 error"; threw = true; }
if (!threw || calls !== 0) throw "setUint32 immutable";
threw = false;
try { immutableView.setFloat16(badOffset, badNumber); } catch (error) { if (!(error instanceof TypeError)) throw "setFloat16 error"; threw = true; }
if (!threw || calls !== 0) throw "setFloat16 immutable";
threw = false;
try { immutableView.setFloat32(badOffset, badNumber); } catch (error) { if (!(error instanceof TypeError)) throw "setFloat32 error"; threw = true; }
if (!threw || calls !== 0) throw "setFloat32 immutable";
threw = false;
try { immutableView.setFloat64(badOffset, badNumber); } catch (error) { if (!(error instanceof TypeError)) throw "setFloat64 error"; threw = true; }
if (!threw || calls !== 0) throw "setFloat64 immutable";
threw = false;
try { immutableView.setBigInt64(badOffset, badBigInt); } catch (error) { if (!(error instanceof TypeError)) throw "setBigInt64 error"; threw = true; }
if (!threw || calls !== 0) throw "setBigInt64 immutable";
threw = false;
try { immutableView.setBigUint64(badOffset, badBigInt); } catch (error) { if (!(error instanceof TypeError)) throw "setBigUint64 error"; threw = true; }
if (!threw || calls !== 0) throw "setBigUint64 immutable";

123;
