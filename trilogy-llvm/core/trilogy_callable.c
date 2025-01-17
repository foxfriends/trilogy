#include <stdlib.h>
#include "trilogy_callable.h"
#include "internal.h"

void trilogy_callable_init(trilogy_value* t, trilogy_callable_value* payload) {
    t->tag = TAG_CALLABLE;
    t->payload = (unsigned long)payload;
}

void trilogy_callable_clone_into(trilogy_value* t, trilogy_callable_value* orig) {
    trilogy_callable_value* callable = malloc(sizeof(trilogy_callable_value));
    callable->tag = orig->tag;
    callable->arity = orig->arity;
    callable->closure = orig->closure; // TODO: this thing needs to be cloneable, probably by being refcounted
    callable->function = orig->function;
    trilogy_callable_init(t, callable);
}

void trilogy_callable_init_func(trilogy_value* t, void* c, void* p) {
    trilogy_callable_value* callable = malloc(sizeof(trilogy_callable_value));
    callable->tag = CALLABLE_FUNCTION;
    callable->arity = 1;
    callable->closure = c;
    callable->function = p;
    trilogy_callable_init(t, callable);
}

void trilogy_callable_init_proc(trilogy_value* t, unsigned int arity, void* c, void* p) {
    trilogy_callable_value* callable = malloc(sizeof(trilogy_callable_value));
    callable->tag = CALLABLE_PROCEDURE;
    callable->arity = arity;
    callable->closure = c;
    callable->function = p;
    trilogy_callable_init(t, callable);
}

void trilogy_callable_init_rule(trilogy_value* t, unsigned int arity, void* c, void* p) {
    trilogy_callable_value* callable = malloc(sizeof(trilogy_callable_value));
    callable->tag = CALLABLE_RULE;
    callable->arity = arity;
    callable->closure = c;
    callable->function = p;
    trilogy_callable_init(t, callable);
}

void trilogy_callable_destroy(trilogy_callable_value* val) {
    if (val->closure != NULL) free(val->closure);
}

trilogy_callable_value* untag_callable(trilogy_value* val) {
    if (val->tag != TAG_CALLABLE) rte("callable", val->tag);
    return assume_callable(val);
}

trilogy_callable_value* assume_callable(trilogy_value* val) {
    return (void*)val->payload;
}

void* untag_function(trilogy_callable_value* val) {
    if (val->tag != CALLABLE_FUNCTION) internal_panic("invalid application of non-function callable");
    return (void*)val->function;
}

void* untag_procedure(trilogy_callable_value* val, unsigned int arity) {
    if (val->tag != CALLABLE_PROCEDURE) internal_panic("invalid call of non-procedure callable");
    if (val->arity != arity) internal_panic("procedure call arity mismatch");
    return (void*)val->function;
}

void* untag_rule(trilogy_callable_value* val, unsigned int arity) {
    if (val->tag != CALLABLE_RULE) internal_panic("invalid call of non-rule callable");
    if (val->arity != arity) internal_panic("rule call arity mismatch");
    return (void*)val->function;
}
