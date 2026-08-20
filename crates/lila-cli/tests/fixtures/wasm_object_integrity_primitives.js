Object.preventExtensions(undefined) === undefined
  && Object.preventExtensions(null) === null
  && Object.preventExtensions(0) === 0
  && Object.preventExtensions(true) === true
  && Object.preventExtensions("abc") === "abc"
  && Object.freeze(undefined) === undefined
  && Object.freeze(null) === null
  && Object.freeze(0) === 0
  && Object.freeze(false) === false
  && Object.freeze(true) === true
  && Object.freeze("abc") === "abc";
