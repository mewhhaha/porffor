var ok = true;

ok = ok && Math.min() === Infinity;
ok = ok && Math.max() === -Infinity;
ok = ok && Math.min(0) === 0;
ok = ok && Math.max(0) === 0;
ok = ok && Math.min(3, 2, 1) === 1;
ok = ok && Math.max(1, 2, 3) === 3;

ok;
