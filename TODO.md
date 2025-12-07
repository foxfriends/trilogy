# TODO

* Runtime new atom (beware Ruby symbol problem)
* `defer` statement
* Rethink `const` (`slot` and `slot mut`?)
* Upgrade `for` to be a folding construct
* Finish FFI
* Build real standard library
* Loosen requirement of pinning identifiers in queries (auto-pin should be better)
* Fix the memory leaks
* Something wrong with `or` patterns when running JIT
* Rethink complex number syntax, make more like normal
* Remove distinction between unary functions and unary procedures
* Make blocks and expressions more interchangeable
* Do a proper standard library design, maybe include a prelude
    * Type-agnostic global functions (similar to core, but safe)
    * Consistent interfaces to individual core standard modules
    * Maybe adjust naming and organization of `core.c`, which is kind of scary conflictable
* Guards on `func`
