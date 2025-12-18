# TODO

* Runtime new atom (beware Ruby symbol problem)
* `defer` statement
* Rethink `const` (`slot` and `slot mut`?)
* Upgrade `for` to be a folding construct
* Finish FFI (foreign types)
* Build real standard library
* Loosen requirement of pinning identifiers in queries (auto-pin should be better)
    * Really: fix queries all over, they're pretty broken
* Fix the memory leaks
* Something wrong with `or` patterns when running JIT
* Rethink complex number syntax, make more like normal
* Make blocks and expressions more interchangeable
* Rethink syntax of cases (`match` and `with`)
    * Add braces like most languages use
    * `else yield` is default fallback for `when`
    * `else end` as default fallback for `match`
    * Remove binding from `else _ then {}` case, as it will be optional and could just be a regular last case 
* Do a proper standard library design, maybe include a prelude
    * Type-agnostic global functions (similar to core, but safe)
    * Consistent interfaces to individual core standard modules
    * Maybe adjust naming and organization of `core.c`, which is kind of scary conflictable
* Guards on `func`
* Improve stack traces and panic messages for debugging
* Additional operators (though this might be resolved by having a prelude):
    * `length` (`#val`)
    * prefix nullary procedure call `!()` (`let x = !() <| many_1 <| char 'x'`)
