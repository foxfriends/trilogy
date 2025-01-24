#pragma once
#include "types.h"

trilogy_value* prepare_closure(unsigned int closure_size);

void trilogy_callable_init(trilogy_value* t, trilogy_callable_value* payload);
void trilogy_callable_init_fn(
    trilogy_value* t, unsigned int closure_size, trilogy_value* c, void* p
);
void trilogy_callable_init_do(
    trilogy_value* t, unsigned int arity, unsigned int closure_size,
    trilogy_value* c, void* p
);
void trilogy_callable_init_qy(
    trilogy_value* t, unsigned int arity, unsigned int closure_size,
    trilogy_value* c, void* p
);
void trilogy_callable_init_func(trilogy_value* t, void* p);
void trilogy_callable_init_proc(trilogy_value* t, unsigned int arity, void* p);
void trilogy_callable_init_rule(trilogy_value* t, unsigned int arity, void* p);
void trilogy_callable_clone_into(trilogy_value*, trilogy_callable_value* orig);
void trilogy_callable_destroy(trilogy_callable_value* val);

trilogy_callable_value* trilogy_callable_untag(trilogy_value* val);
trilogy_callable_value* trilogy_callable_assume(trilogy_value* val);

void* trilogy_function_untag(trilogy_callable_value* val);
void* trilogy_procedure_untag(trilogy_callable_value* val, unsigned int arity);
void* trilogy_rule_untag(trilogy_callable_value* val, unsigned int arity);
