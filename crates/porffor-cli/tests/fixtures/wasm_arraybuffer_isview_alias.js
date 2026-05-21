var isView = ArrayBuffer.isView;

if (isView() !== false) throw "undefined arg";
if (isView({}) !== false) throw "plain object";
if (isView(new ArrayBuffer(1)) !== false) throw "arraybuffer";
if (isView(new DataView(new ArrayBuffer(1), 0, 0)) !== true) throw "dataview";
