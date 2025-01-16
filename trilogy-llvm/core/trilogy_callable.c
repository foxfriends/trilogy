#include <stdlib.h>
#include "trilogy_callable.h"
#include "internal.h"

trilogy_callable_value* untag_callable(trilogy_value* val) {
    if (val->tag != TAG_CALLABLE) rte("callable", val->tag);
    return assume_callable(val);
}

trilogy_callable_value* assume_callable(trilogy_value* val) {
    return (void*)val->payload;
}

void* untag_function(trilogy_callable_value* val) {
    if (val->tag != CALLABLE_FUNCTION) panic("invalid application of non-function callable");
    return (void*)val->function;
}

void* untag_procedure(trilogy_callable_value* val, unsigned int arity) {
    if (val->tag != CALLABLE_PROCEDURE) panic("invalid call of non-procedure callable");
    if (val->arity != arity) panic("procedure call arity mismatch");
    return (void*)val->function;
}

void* untag_rule(trilogy_callable_value* val, unsigned int arity) {
    if (val->tag != CALLABLE_RULE) panic("invalid call of non-rule callable");
    if (val->arity != arity) panic("rule call arity mismatch");
    return (void*)val->function;
}

static trilogy_value trilogy_callable(trilogy_callable_value* payload) {
    trilogy_value t = { .tag = TAG_CALLABLE, .payload = (unsigned long)payload };
    return t;
}

trilogy_value trilogy_function(void* c, void* p) {
    trilogy_callable_value* callable = malloc(sizeof(trilogy_callable_value));
    callable->tag = CALLABLE_FUNCTION;
    callable->closure = NULL;
    callable->arity = 1;
    callable->closure = c;
    callable->function = p;
    return trilogy_callable(callable);
}

trilogy_value trilogy_procedure(unsigned int arity, void* c, void* p) {
    trilogy_callable_value* callable = malloc(sizeof(trilogy_callable_value));
    callable->tag = CALLABLE_PROCEDURE;
    callable->closure = NULL;
    callable->arity = arity;
    callable->closure = c;
    callable->function = p;
    return trilogy_callable(callable);
}

trilogy_value trilogy_rule(unsigned int arity, void* c, void* p) {
    trilogy_callable_value* callable = malloc(sizeof(trilogy_callable_value));
    callable->tag = CALLABLE_RULE;
    callable->closure = NULL;
    callable->arity = arity;
    callable->closure = c;
    callable->function = p;
    return trilogy_callable(callable);
}

void destroy_callable(trilogy_callable_value* val) {
    if (val->closure != NULL) free(val->closure);
}
