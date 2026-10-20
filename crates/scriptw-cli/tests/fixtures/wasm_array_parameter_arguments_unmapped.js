var result = false;

function dstr(a, [, b = 3, ...rest], [[nested]]) {
  arguments[0] = 2;
  result = a === 1
    && b === 3
    && rest.length === 2
    && rest[0] === 4
    && rest[1] === 5
    && nested === 6;
}

dstr(1, [0, undefined, 4, 5], [[6]]);
result;
