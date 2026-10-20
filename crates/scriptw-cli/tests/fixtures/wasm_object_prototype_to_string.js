Object.prototype.toString.call([]) === "[object Array]"
  && Object.prototype.toString.call({}) === "[object Object]"
  && Object.prototype.toString.call(function(){}) === "[object Function]"
  && Object.prototype.toString.call(undefined) === "[object Undefined]"
  && Object.prototype.toString.call(null) === "[object Null]";
