#pragma once
#include "types.h"

void trilogy_callable_init(trilogy_value* t, trilogy_callable_value* payload);
void trilogy_callable_init_func(trilogy_value* t, void* c, void* p);
void trilogy_callable_init_proc(
    trilogy_value* t, unsigned int arity, void* c, void* p
);
void trilogy_callable_init_rule(
    trilogy_value* t, unsigned int arity, void* c, void* p
);
void trilogy_callable_clone_into(trilogy_value*, trilogy_callable_value* orig);
void trilogy_callable_destroy(trilogy_callable_value* val);

trilogy_callable_value* trilogy_callable_untag(trilogy_value* val);
trilogy_callable_value* trilogy_callable_assume(trilogy_value* val);

void* trilogy_function_untag(trilogy_callable_value* val);
void* trilogy_procedure_untag(trilogy_callable_value* val, unsigned int arity);
void* trilogy_rule_untag(trilogy_callable_value* val, unsigned int arity);
