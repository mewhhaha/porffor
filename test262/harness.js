// Portions of this file are adapted from Test262 (https://github.com/tc39/test262)
// Test262 is BSD-3-Clause licensed; see the upstream LICENSE file

/// sta.js
function Test262Error(message) {
  this.message = message || '';
  this.name = 'Test262Error';
}

Test262Error.prototype.toString = function () {
  return 'Test262Error: ' + this.message;
};

Test262Error.thrower = function (message) {
  throw new Test262Error(message);
};

function $DONOTEVALUATE() {
  throw 'Test262: This statement should not be evaluated.';
}

var $262 = {
  global: globalThis,
  gc() {
    if (typeof gc === 'function') gc();
  },
  detachArrayBuffer(buffer) {
    if (typeof __porfDetachArrayBuffer === 'function') {
      return __porfDetachArrayBuffer(buffer);
    }
  },
  getGlobal(name) {
    return globalThis[name];
  },
  evalScript(code) {
    return (0, eval)(String(code));
  },
  createRealm() {
    return {
      global: globalThis,
      evalScript(code) {
        return (0, eval)(String(code));
      },
      getGlobal(name) {
        return globalThis[name];
      },
      destroy() {}
    };
  },
  destroy() {},
  agent: {
    _agents: [],
    _reports: [],
    _pendingWaiters: [],
    _queueWaiter(waiter) {
      this._pendingWaiters.push(waiter);
    },
    _resolveWaitersFor(typedArray, index, count) {
      var awakened = 0;
      for (var waiterIndex = 0; waiterIndex < this._pendingWaiters.length; waiterIndex += 1) {
        var waiter = this._pendingWaiters[waiterIndex];
        if (waiter.status !== null) continue;
        if (waiter.buffer !== typedArray.buffer) continue;
        if (waiter.index !== index) continue;
        if (count !== Infinity && awakened >= count) continue;
        waiter.status = "ok";
        this._reports.push("ok");
        awakened += 1;
      }
      return awakened;
    },
    _flushTimedOutWaiters() {
      for (var waiterIndex = 0; waiterIndex < this._pendingWaiters.length; waiterIndex += 1) {
        var waiter = this._pendingWaiters[waiterIndex];
        if (waiter.status !== null) continue;
        waiter.status = "timed-out";
        this._reports.push("timed-out");
      }
    },
    _run(source) {
      var state = {
        callback: null,
        left: false
      };

      function coerceImmediateTimeout(timeout) {
        if (timeout === undefined) return Infinity;
        var number = Number(timeout);
        if (!Number.isFinite(number) || number < 0) {
          return 0;
        }
        return number;
      }

      function fakeWait(typedArray, index, value, timeout) {
        var actual = Atomics.load(typedArray, index);
        if (!Object.is(actual, value)) {
          return "not-equal";
        }
        var normalizedTimeout = coerceImmediateTimeout(timeout);
        if (normalizedTimeout === 0) {
          return "timed-out";
        }
        return {
          __porfWaiter: true,
          buffer: typedArray.buffer,
          index: index,
          expected: value,
          timeout: normalizedTimeout,
          status: null
        };
      }

      var fakeAtomics = Object.create(Atomics);
      fakeAtomics.notify = function notify(typedArray, index, count) {
        var normalizedCount = count === undefined ? Infinity : Math.max(0, Math.trunc(Number(count)));
        return $262.agent._resolveWaitersFor(typedArray, index, normalizedCount);
      };
      fakeAtomics.wake = fakeAtomics.notify;
      fakeAtomics.wait = function wait(typedArray, index, value, timeout) {
        return fakeWait(typedArray, index, value, timeout);
      };
      fakeAtomics.waitAsync = function waitAsync(typedArray, index, value, timeout) {
        return { async: false, value: fakeWait(typedArray, index, value, timeout) };
      };

      var agentApi = {
        start() {
          throw new Test262Error('nested agent.start is not supported');
        },
        broadcast() {
          throw new Test262Error('nested agent.broadcast is not supported');
        },
        receiveBroadcast(callback) {
          state.callback = callback;
        },
        report(value) {
          if (value && value.__porfWaiter === true) {
            $262.agent._queueWaiter(value);
            return;
          }
          $262.agent._reports.push(String(value));
        },
        getReport() {
          if (!$262.agent._reports.length) {
            $262.agent._flushTimedOutWaiters();
          }
          return $262.agent._reports.length ? $262.agent._reports.shift() : null;
        },
        sleep() {},
        monotonicNow() {
          return Date.now();
        },
        leaving() {
          state.left = true;
        }
      };

      var local262 = Object.create($262);
      local262.agent = agentApi;
      new Function("$262", "Atomics", source)(local262, fakeAtomics);
      return state;
    },
    start(source) {
      this._agents.push(this._run(String(source)));
    },
    broadcast(buffer) {
      var pending = [];
      for (var index = 0; index < this._agents.length; index++) {
        var state = this._agents[index];
        if (!state || typeof state.callback !== "function") continue;
        try {
          var result = state.callback(buffer);
          if (result && typeof result.then === "function") {
            pending.push(result.catch(function(error) {
              $262.agent._reports.push(String(error));
            }));
          }
        } catch (error) {
          this._reports.push(String(error));
        }
      }
      if (pending.length) {
        return Promise.all(pending).then(function() {
          return pending.length;
        });
      }
      return this._agents.length;
    },
    receiveBroadcast() {
      throw new Test262Error('receiveBroadcast may only be used inside agent.start');
    },
    report() {},
    getReport() {
      if (!this._reports.length) {
        this._flushTimedOutWaiters();
      }
      return this._reports.length ? this._reports.shift() : null;
    },
    sleep() {},
    monotonicNow() {
      return Date.now();
    },
    leaving() {}
  }
};

(function installAtomicsHostShim() {
  if (typeof Atomics !== "object" || Atomics === null) {
    return;
  }

  function normalizeNotifyCount(count) {
    if (count === undefined) {
      return Infinity;
    }
    var number = Number(count);
    if (!Number.isFinite(number)) {
      return number > 0 ? Infinity : 0;
    }
    return Math.max(0, Math.trunc(number));
  }

  var originalWait = typeof Atomics.wait === "function" ? Atomics.wait : null;
  var originalWaitAsync = typeof Atomics.waitAsync === "function" ? Atomics.waitAsync : null;
  var originalNotify = typeof Atomics.notify === "function" ? Atomics.notify : null;
  var originalWake = typeof Atomics.wake === "function" ? Atomics.wake : null;
  function installAtomicsMethod(name, value) {
    try {
      Object.defineProperty(Atomics, name, {
        value: value,
        writable: true,
        enumerable: false,
        configurable: true
      });
    } catch (_error) {
      Atomics[name] = value;
    }
  }

  installAtomicsMethod("wait", function wait(typedArray, index, value, timeout) {
    var actual = Atomics.load(typedArray, index);
    if (!Object.is(actual, value)) {
      return "not-equal";
    }
    if (originalWait !== null) {
      try {
        return originalWait.call(Atomics, typedArray, index, value, timeout);
      } catch (_error) {
        return "timed-out";
      }
    }
    return "timed-out";
  });

  installAtomicsMethod("waitAsync", function waitAsync(typedArray, index, value, timeout) {
    var result = Atomics.wait(typedArray, index, value, timeout);
    return { async: false, value: result };
  });

  installAtomicsMethod("notify", function notify(typedArray, index, count) {
    var normalizedCount = normalizeNotifyCount(count);
    var awakened = $262.agent._resolveWaitersFor(typedArray, index, normalizedCount);
    if (awakened !== 0) {
      return awakened;
    }
    if (originalNotify !== null) {
      try {
        return originalNotify.call(Atomics, typedArray, index, count);
      } catch (_error) {
        return 0;
      }
    }
    return 0;
  });

  if (originalWake !== null) {
    installAtomicsMethod("wake", function wake(typedArray, index, count) {
      return Atomics.notify(typedArray, index, count);
    });
  }
})();
