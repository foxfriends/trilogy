# TODO

* Runtime new atom (beware Ruby symbol problem)
* `defer` statement
* Upgrade `for` to be a folding construct
* Finish FFI (foreign types)
* Loosen requirement of pinning identifiers in queries (auto-pin should be better)
    * Really: fix queries all over, they're pretty broken
* Fix the memory leaks
* Something wrong with `or` patterns when running JIT
* Make blocks and expressions more interchangeable:
    * Block is a sequence expression
    * When a block allows for disambiguation, allow `then` keyword to start an expression (e.g. `if`, `case`, `when`)
* Do a proper standard library design, maybe include a prelude
    * Type-agnostic global functions (similar to core, but safe)
    * Consistent interfaces to individual core standard modules
    * Maybe adjust naming and organization of `core.c`, which is kind of scary conflictable
* Guards on `func`
* Improve error messages:
    * Eliminate places where panics are visible to the end user; should be covered by `yield`/`end` (rewrite spec I guess)
    * Review error handling in scanner/parser/IR, so all are nicely reported; maybe reimplement this using trait objects so it's not so spread out
    * Implement stack traces and better messages in panics
* Additional operators (though this might be resolved by having a prelude):
    * `length` (`#val`)
    * prefix nullary procedure call `!()` (`let x = !() <| many_1 <| char 'x'`)
* Steal `use` from Gleam:
    * Unary: `let x using array::each [1, 2, 3]; ...` -> `array::each [1, 2, 3] do(x) {...}`
    * Nullary: `using times 3` -> `times 3 do() {...}`
* Consider nullary type definitions (is that just `ty` defs inside a procedure?)
* Swap order of arguments to the reducer function in fold/reduce (`fn item acc. ...`)
* Partially applied binary operators (e.g. `(< 3)` or `(4 :)`)
