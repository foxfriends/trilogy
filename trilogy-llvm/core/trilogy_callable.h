#pragma once
#include "types.h"

trilogy_value trilogy_function(void* c, void* p);
trilogy_value trilogy_procedure(unsigned int arity, void* c, void* p);
trilogy_value trilogy_rule(unsigned int arity, void* c, void* p);
trilogy_callable_value* untag_callable(trilogy_value* val);
trilogy_callable_value* assume_callable(trilogy_value* val);
void destroy_callable(trilogy_callable_value* val);

void* untag_function(trilogy_callable_value* val);
void* untag_procedure(trilogy_callable_value* val, unsigned int arity);
void* untag_rule(trilogy_callable_value* val, unsigned int arity);
