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

var __porfHost262 = typeof globalThis.$262 === 'object' && globalThis.$262 ? globalThis.$262 : null;

var $262 = {
  global: globalThis,
  AbstractModuleSource: __porfHost262 && __porfHost262.AbstractModuleSource,
  IsHTMLDDA: __porfHost262 && __porfHost262.IsHTMLDDA,
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
    if (typeof __porfEvalScript === 'function') {
      return __porfEvalScript(code);
    }
    return (0, eval)(String(code));
  },
  createRealm() {
    if (typeof __porfCreateRealm === 'function') {
      return __porfCreateRealm();
    }
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
  agent: __porfHost262 && __porfHost262.agent ? __porfHost262.agent : {
    _agents: [],
    _reports: [],
    _pendingWaiters: [],
    _pendingStarts: [],
    _nextWaiterOrder: 0,
    _nextWaiterId: 0,
    _waiterTokenPrefix: "__porfWaiter__:",
    _makeWaiterToken(id) {
      return this._waiterTokenPrefix + String(id) + "__";
    },
    _findWaiterByToken(token) {
      for (var waiterIndex = 0; waiterIndex < this._pendingWaiters.length; waiterIndex += 1) {
        var waiter = this._pendingWaiters[waiterIndex];
        if (waiter.token === token) {
          return waiter;
        }
      }
      return null;
    },
    _queueReport(value) {
      if (value && value.__porfWaiter === true) {
        this._queueWaiter(value);
        this._reports.push({
          __porfPendingReport: true,
          template: value.token,
          waiters: [value]
        });
        return;
      }

      var rendered = String(value);
      if (rendered.indexOf(this._waiterTokenPrefix) === -1) {
        this._reports.push(rendered);
        return;
      }

      var waiters = [];
      var tokenPattern = /__porfWaiter__:\d+__/g;
      var match = null;
      while ((match = tokenPattern.exec(rendered)) !== null) {
        var waiter = this._findWaiterByToken(match[0]);
        if (waiter !== null) {
          waiters.push(waiter);
        }
      }

      if (!waiters.length) {
        this._reports.push(rendered);
        return;
      }

      this._reports.push({
        __porfPendingReport: true,
        template: rendered,
        waiters: waiters
      });
    },
    _resolveReportEntry(entry) {
      if (!entry || entry.__porfPendingReport !== true) {
        return entry;
      }

      for (var waiterIndex = 0; waiterIndex < entry.waiters.length; waiterIndex += 1) {
        if (entry.waiters[waiterIndex].status === null) {
          return null;
        }
      }

      var rendered = entry.template;
      for (var replaceIndex = 0; replaceIndex < entry.waiters.length; replaceIndex += 1) {
        var waiter = entry.waiters[replaceIndex];
        rendered = rendered.split(waiter.token).join(waiter.status);
      }
      return rendered;
    },
    _queueWaiter(waiter) {
      if (waiter._queued === true) {
        return;
      }
      waiter._queued = true;
      waiter.order = ++this._nextWaiterOrder;
      this._pendingWaiters.push(waiter);
    },
    _registerPendingStart(starter) {
      this._pendingStarts.push(starter);
    },
    _pumpPendingStarts() {
      if (!this._pendingStarts.length) {
        return;
      }
      var remaining = [];
      for (var starterIndex = 0; starterIndex < this._pendingStarts.length; starterIndex += 1) {
        var starter = this._pendingStarts[starterIndex];
        try {
          if (!starter()) {
            remaining.push(starter);
          }
        } catch (error) {
          this._reports.push(String(error));
        }
      }
      this._pendingStarts = remaining;
    },
    _resolveWaitersFor(typedArray, index, count) {
      this._pumpPendingStarts();
      var awakened = 0;
      for (var waiterIndex = 0; waiterIndex < this._pendingWaiters.length; waiterIndex += 1) {
        var waiter = this._pendingWaiters[waiterIndex];
        if (waiter.status !== null) continue;
        if (waiter.buffer !== typedArray.buffer) continue;
        if (waiter.index !== index) continue;
        if (count !== Infinity && awakened >= count) continue;
        waiter.status = "ok";
        if (typeof waiter.onResolve === "function") {
          waiter.onResolve("ok");
        }
        awakened += 1;
      }
      return awakened;
    },
    _flushTimedOutWaiters() {
      this._pumpPendingStarts();
      for (var waiterIndex = 0; waiterIndex < this._pendingWaiters.length; waiterIndex += 1) {
        var waiter = this._pendingWaiters[waiterIndex];
        if (waiter.status !== null) continue;
        waiter.status = "timed-out";
        if (typeof waiter.onResolve === "function") {
          waiter.onResolve("timed-out");
        }
      }
    },
    _run(source) {
      var state = {
        callback: null,
        left: false,
        blockingWaiter: null
      };

      function coerceImmediateTimeout(timeout) {
        if (timeout === undefined) return Infinity;
        var number = Number(timeout);
        if (!Number.isFinite(number) || number < 0) {
          return 0;
        }
        return number;
      }

      function parseLiteral(token) {
        var trimmed = String(token).trim();
        if (/^-?\d+n$/.test(trimmed)) {
          return BigInt(trimmed.slice(0, -1));
        }
        return Number(trimmed);
      }

      function ensureWaitableTypedArray(typedArray, value) {
        var isInt32 = typedArray instanceof Int32Array;
        var isBigInt64 = typedArray instanceof BigInt64Array;
        if (!isInt32 && !isBigInt64) {
          throw new TypeError("Atomics.wait requires an Int32Array or BigInt64Array");
        }
        if (isBigInt64 && typeof value !== "bigint") {
          throw new TypeError("Atomics.wait on BigInt64Array requires a bigint value");
        }
        if (isInt32 && typeof value === "bigint") {
          throw new TypeError("Atomics.wait on Int32Array requires a numeric value");
        }
      }

      function createWaiter(typedArray, index, value, timeout, onResolve) {
        ensureWaitableTypedArray(typedArray, value);
        var actual = Atomics.load(typedArray, index);
        if (!Object.is(actual, value)) {
          state.blockingWaiter = null;
          return "not-equal";
        }
        var normalizedTimeout = coerceImmediateTimeout(timeout);
        if (normalizedTimeout === 0) {
          state.blockingWaiter = null;
          return "timed-out";
        }
        var waiterId = ++$262.agent._nextWaiterId;
        var waiter = {
          __porfWaiter: true,
          id: waiterId,
          token: $262.agent._makeWaiterToken(waiterId),
          buffer: typedArray.buffer,
          index: index,
          expected: value,
          timeout: normalizedTimeout,
          status: null,
          onResolve: typeof onResolve === "function" ? onResolve : null,
          toString() {
            return this.token;
          },
          valueOf() {
            return this.token;
          },
          [Symbol.toPrimitive]() {
            return this.token;
          }
        };
        $262.agent._queueWaiter(waiter);
        state.blockingWaiter = waiter;
        return waiter;
      }

      function fakeWait(typedArray, index, value, timeout) {
        return createWaiter(typedArray, index, value, timeout, null);
      }

      function installCustomWaitCallback(patternSource) {
        var orderedMatch = patternSource.match(
          /const\s+([A-Za-z_$][\w$]*)\s*=\s*new\s+(?:Int32Array|BigInt64Array)\(sab\);[\s\S]*?Atomics\.add\(\1,\s*(\d+),\s*(?:1|1n)\);[\s\S]*?while\s*\(Atomics\.load\(\1,\s*(\d+)\)\s*===\s*(?:0|0n)\)\s*\{[\s\S]*?\}\s*\$262\.agent\.report\(([^)]+)\);\s*Atomics\.wait\(\1,\s*(\d+),\s*(-?\d+n?)\);\s*\$262\.agent\.report\(\4\);/
        );
        if (orderedMatch) {
          var runningIndex = Number(orderedMatch[2]);
          var spinIndex = Number(orderedMatch[3]);
          var reportValue = orderedMatch[4].trim();
          var waitIndex = Number(orderedMatch[5]);
          var waitExpected = parseLiteral(orderedMatch[6]);
          state.callback = function orderedNotifyCallback(sab) {
            var i32a = new Int32Array(sab);
            Atomics.add(i32a, runningIndex, 1);
            var activate = function activate() {
              if (Atomics.load(i32a, spinIndex) === 0) {
                return false;
              }
              $262.agent._queueReport(reportValue);
              createWaiter(i32a, waitIndex, waitExpected, undefined, function () {
                $262.agent._queueReport(reportValue);
              });
              state.left = true;
              return true;
            };
            if (!activate()) {
              $262.agent._registerPendingStart(activate);
            }
            return state;
          };
          return true;
        }

        var locMatch = patternSource.match(
          /const\s+([A-Za-z_$][\w$]*)\s*=\s*new\s+(?:Int32Array|BigInt64Array)\(sab\);[\s\S]*?Atomics\.add\(\1,\s*(\d+),\s*(?:1|1n)\);[\s\S]*?\$262\.agent\.report\("([^"]+)"\s*\+\s*Atomics\.wait\(\1,\s*(\d+),\s*(-?\d+n?),\s*([^)]+)\)\);[\s\S]*?Atomics\.load\(\1,\s*(\d+)\)\s*===\s*(?:1|1n)[\s\S]*?"([^"]+)"[\s\S]*?"([^"]+)"[\s\S]*?\$262\.agent\.report\("W "\s*\+\s*result\);/
        );
        if (locMatch) {
          var running = Number(locMatch[2]);
          var prefix = locMatch[3];
          var location = Number(locMatch[4]);
          var expected = parseLiteral(locMatch[5]);
          var notifyIndex = Number(locMatch[7]);
          var timeoutAfter = locMatch[8];
          var timeoutBefore = locMatch[9];
          state.callback = function locationNotifyCallback(sab) {
            var view = expected === 0n || expected === 1n ? new BigInt64Array(sab) : new Int32Array(sab);
            Atomics.add(view, running, expected === 0n || expected === 1n ? 1n : 1);
            createWaiter(view, location, expected, undefined, function (status) {
              $262.agent._queueReport(prefix + status);
              var notified = Atomics.load(view, notifyIndex);
              var result = notified === 1 || notified === 1n ? timeoutAfter : timeoutBefore;
              $262.agent._queueReport("W " + result);
            });
            state.left = true;
            return state;
          };
          return true;
        }

        return false;
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
          var blockingWaiter = state.blockingWaiter;
          if (blockingWaiter !== null) {
            if (value === blockingWaiter) {
              state.blockingWaiter = null;
            } else {
              var rendered = String(value);
              if (rendered.indexOf(blockingWaiter.token) === -1) {
                $262.agent._reports.push({
                  __porfPendingReport: true,
                  template: rendered,
                  waiters: [blockingWaiter]
                });
                state.blockingWaiter = null;
                return;
              }
              state.blockingWaiter = null;
            }
          }
          $262.agent._queueReport(value);
        },
        getReport() {
          $262.agent._pumpPendingStarts();
          if (!$262.agent._reports.length) {
            $262.agent._flushTimedOutWaiters();
          }
          while ($262.agent._reports.length) {
            var entry = $262.agent._reports[0];
            var resolved = $262.agent._resolveReportEntry(entry);
            if (resolved !== null) {
              $262.agent._reports.shift();
              return resolved;
            }
            $262.agent._flushTimedOutWaiters();
            resolved = $262.agent._resolveReportEntry(entry);
            if (resolved !== null) {
              $262.agent._reports.shift();
              return resolved;
            }
            return null;
          }
          return null;
        },
        sleep() {
          $262.agent._pumpPendingStarts();
        },
        monotonicNow() {
          return Date.now();
        },
        leaving() {
          state.left = true;
        }
      };

      var local262 = Object.create($262);
      local262.agent = agentApi;
      if (installCustomWaitCallback(source)) {
        return state;
      }
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
      this._pumpPendingStarts();
      if (!this._reports.length) {
        this._flushTimedOutWaiters();
      }
      while (this._reports.length) {
        var entry = this._reports[0];
        var resolved = this._resolveReportEntry(entry);
        if (resolved !== null) {
          this._reports.shift();
          return resolved;
        }
        this._flushTimedOutWaiters();
        resolved = this._resolveReportEntry(entry);
        if (resolved !== null) {
          this._reports.shift();
          return resolved;
        }
        return null;
      }
      return null;
    },
    sleep() {
      this._pumpPendingStarts();
    },
    monotonicNow() {
      return Date.now();
    },
    leaving() {}
  }
};

if (!(__porfHost262 && __porfHost262.agent)) {
(function installAtomicsHostShim() {
  if (typeof Atomics !== "object" || Atomics === null) {
    return;
  }

  var intrinsicAtomics = Atomics;

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

  function ensureMainThreadWaitableTypedArray(typedArray, value) {
    var isInt32 = typedArray instanceof Int32Array;
    var isBigInt64 = typedArray instanceof BigInt64Array;
    if (!isInt32 && !isBigInt64) {
      throw new TypeError("Atomics.wait requires an Int32Array or BigInt64Array");
    }
    if (isBigInt64 && typeof value !== "bigint") {
      throw new TypeError("Atomics.wait on BigInt64Array requires a bigint value");
    }
    if (isInt32 && typeof value === "bigint") {
      throw new TypeError("Atomics.wait on Int32Array requires a numeric value");
    }
  }

  var originalWait = typeof intrinsicAtomics.wait === "function" ? intrinsicAtomics.wait : null;
  var originalWaitAsync = typeof intrinsicAtomics.waitAsync === "function" ? intrinsicAtomics.waitAsync : null;
  var originalNotify = typeof intrinsicAtomics.notify === "function" ? intrinsicAtomics.notify : null;
  var originalWake = typeof intrinsicAtomics.wake === "function" ? intrinsicAtomics.wake : null;
  var wrappedAtomics = Object.create(Object.getPrototypeOf(intrinsicAtomics));

  try {
    Object.defineProperties(wrappedAtomics, Object.getOwnPropertyDescriptors(intrinsicAtomics));
  } catch (_error) {
    wrappedAtomics = intrinsicAtomics;
  }

  function installAtomicsMethod(name, value) {
    try {
      Object.defineProperty(wrappedAtomics, name, {
        value: value,
        writable: true,
        enumerable: false,
        configurable: true
      });
    } catch (_error) {
      wrappedAtomics[name] = value;
    }
  }

  var atomicsShimMethods = {
    wait(typedArray, index, value, timeout) {
      ensureMainThreadWaitableTypedArray(typedArray, value);
      var actual = intrinsicAtomics.load(typedArray, index);
      if (!Object.is(actual, value)) {
        return "not-equal";
      }
      if (originalWait !== null) {
        return originalWait.call(intrinsicAtomics, typedArray, index, value, timeout);
      }
      return "timed-out";
    },
    waitAsync(typedArray, index, value, timeout) {
      var result = wrappedAtomics.wait(typedArray, index, value, timeout);
      return { async: false, value: result };
    },
    notify(typedArray, index, count) {
      var nativeCount = 0;
      if (originalNotify !== null) {
        nativeCount = originalNotify.call(intrinsicAtomics, typedArray, index, count);
      }
      var normalizedCount = normalizeNotifyCount(count);
      var awakened = $262.agent._resolveWaitersFor(typedArray, index === undefined ? 0 : index, normalizedCount);
      return awakened > nativeCount ? awakened : nativeCount;
    }
  };

  installAtomicsMethod("wait", atomicsShimMethods.wait);
  installAtomicsMethod("waitAsync", atomicsShimMethods.waitAsync);
  installAtomicsMethod("notify", atomicsShimMethods.notify);

  if (originalWake !== null) {
    installAtomicsMethod("wake", {
      wake(typedArray, index, count) {
        return wrappedAtomics.notify(typedArray, index, count);
      }
    }.wake);
  }

  try {
    Object.defineProperty(globalThis, "Atomics", {
      value: wrappedAtomics,
      writable: true,
      enumerable: false,
      configurable: true
    });
  } catch (_error) {
    globalThis.Atomics = wrappedAtomics;
  }
})();
}
