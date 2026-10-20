const first = import('./bad.js'); const second = import('./bad.js'); if (first === second) throw 'reused import promise';
let count = 0; for (const promise of [first, second]) { try { await promise; } catch (error) { if (error !== undefined) throw 'replaced rejection'; count++; } }
if (count !== 2) throw 'missing rejection'; print('undefined import rejection');