# TODO

* Runtime new atom (beware Ruby symbol problem)
* `defer` statement
* Upgrade `for` to be a folding construct
* Finish FFI (foreign types)
* Loosen requirement of pinning identifiers in queries (auto-pin should be better)
    * Really: fix queries all over, they're pretty broken
* Fix the memory leaks
* Something wrong with `or` patterns when running JIT
* Allow `with {}` (with a block)
* Make blocks and expressions more interchangeable
* Do a proper standard library design, maybe include a prelude
    * Type-agnostic global functions (similar to core, but safe)
    * Consistent interfaces to individual core standard modules
    * Maybe adjust naming and organization of `core.c`, which is kind of scary conflictable
* Guards on `func`
* Improve stack traces and panic messages for debugging
* Additional operators (though this might be resolved by having a prelude):
    * `length` (`#val`)
    * prefix nullary procedure call `!()` (`let x = !() <| many_1 <| char 'x'`)
* Steal `use` from Gleam:
    * Unary: `let x using array::each [1, 2, 3]; ...` -> `array::each [1, 2, 3] do(x) {...}`
    * Nullary: `using times 3` -> `times 3 do() {...}`
* Consider nullary type definitions (is that just `ty` defs inside a procedure?)
