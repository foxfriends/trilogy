# TODO

* Runtime new atom (beware Ruby symbol problem)
* `defer` statement
* Upgrade `for` to be a folding construct
 
```trilogy
# let [1, 2, 3] = from list = [] for vals(x) { [...list, x] }
# let [1, 2, 3] = for vals(x) into list = [] { [...list, x] }
let [1, 2, 3] = for vals(x) where list = [] { [...list, x] }
# let [1, 2, 3] = for vals(x) let list = [] { [...list, x] }

next
break
continue = break << next
```

* Finish FFI (foreign types)
* Loosen requirement of pinning identifiers in queries (auto-pin should be better)
    * Really: fix queries all over, they're pretty broken
* Fix the memory leaks
* Something wrong with `or` patterns when running JIT
* Do a proper standard library design, maybe include a prelude
    * Type-agnostic global functions (similar to core, but safe)
    * Consistent interfaces to individual core standard modules
    * Maybe adjust naming and organization of `core.c`, which is kind of scary conflictable
* Improve error messages:
    * Eliminate places where panics are visible to the end user; should be covered by `yield`/`end` (rewrite spec I guess)
    * Review error handling in scanner/parser/IR, so all are nicely reported; maybe reimplement this using trait objects so it's not so spread out
    * Implement stack traces and better messages in panics
* Additional operators (though this might be resolved by having a prelude):
    * `length` (`#val`)
    * prefix nullary procedure call `!()` (`let x = !() <| many_1 <| char 'x'`)
* Consider nullary type definitions (is that just `ty` defs inside a procedure?)
* Partially applied binary operators (e.g. `(< 3)` or `(4 :)`)
