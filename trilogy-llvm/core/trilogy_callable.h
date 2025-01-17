#pragma once
#include "types.h"

void trilogy_callable_init(trilogy_value* t, trilogy_callable_value* payload);
void trilogy_callable_init_func(trilogy_value* t, void* c, void* p);
void trilogy_callable_init_proc(trilogy_value* t, unsigned int arity, void* c, void* p);
void trilogy_callable_init_rule(trilogy_value* t, unsigned int arity, void* c, void* p);
void trilogy_callable_clone_into(trilogy_value*, trilogy_callable_value* orig);
void trilogy_callable_destroy(trilogy_callable_value* val);

trilogy_callable_value* untag_callable(trilogy_value* val);
trilogy_callable_value* assume_callable(trilogy_value* val);

void* untag_function(trilogy_callable_value* val);
void* untag_procedure(trilogy_callable_value* val, unsigned int arity);
void* untag_rule(trilogy_callable_value* val, unsigned int arity);
