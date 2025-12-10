#pragma once
#include "types.h"
#include <stdint.h>

#define NO_CLOSURE 0

trilogy_callable_value*
trilogy_callable_init(trilogy_value* t, trilogy_callable_value* payload);
trilogy_callable_value* trilogy_callable_init_do(
    trilogy_value* t, uint32_t arity, trilogy_value* closure, void* p
);
trilogy_callable_value* trilogy_callable_init_qy(
    trilogy_value* t, uint32_t arity, trilogy_value* closure, void* p
);

trilogy_callable_value* trilogy_callable_init_root(trilogy_value* t, void* p);

trilogy_callable_value*
trilogy_callable_init_proc(trilogy_value* t, uint32_t arity, void* p);
trilogy_callable_value*
trilogy_callable_init_rule(trilogy_value* t, uint32_t arity, void* p);
trilogy_callable_value* trilogy_callable_init_cont(
    trilogy_value* t, trilogy_value* return_to /* moved */,
    trilogy_value* yield_to /* moved */, trilogy_value* closure /* moved */,
    void* p
);
void trilogy_callable_clone_into(trilogy_value*, trilogy_callable_value* orig);
void trilogy_callable_destroy(trilogy_callable_value* val);

trilogy_array_value*
trilogy_callable_closure_into(trilogy_value*, trilogy_callable_value* orig);
void trilogy_callable_return_to_into(
    trilogy_value*, trilogy_callable_value* orig
);
void trilogy_callable_yield_to_into(
    trilogy_value*, trilogy_callable_value* orig
);

void trilogy_callable_promote(
    trilogy_value* tv, trilogy_value* return_to, trilogy_value* yield_to
);

trilogy_callable_value* trilogy_callable_untag(trilogy_value* val);
trilogy_callable_value* trilogy_callable_assume(trilogy_value* val);

void* trilogy_procedure_untag(trilogy_callable_value* val, uint32_t arity);
void* trilogy_rule_untag(trilogy_callable_value* val, uint32_t arity);
void* trilogy_continuation_untag(trilogy_callable_value* val);
