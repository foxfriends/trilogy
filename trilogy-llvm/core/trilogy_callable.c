#include "trilogy_callable.h"
#include "internal.h"
#include "trilogy_array.h"
#include <assert.h>

#define NO_CLOSURE 0

void trilogy_callable_init(trilogy_value* t, trilogy_callable_value* payload) {
    assert(t->tag == TAG_UNDEFINED);
    t->tag = TAG_CALLABLE;
    t->payload = (unsigned long)payload;
}

void trilogy_callable_clone_into(
    trilogy_value* t, trilogy_callable_value* orig
) {
    trilogy_callable_value* callable =
        malloc_safe(sizeof(trilogy_callable_value));
    callable->tag = orig->tag;
    callable->arity = orig->arity;
    callable->closure = orig->closure;
    if (callable->closure != NO_CLOSURE) callable->closure->rc += 1;
    callable->function = orig->function;
    trilogy_callable_init(t, callable);
}

void trilogy_callable_init_func(
    trilogy_value* t, trilogy_array_value* c, void* p
) {
    trilogy_callable_value* callable =
        malloc_safe(sizeof(trilogy_callable_value));
    callable->tag = CALLABLE_FUNCTION;
    callable->arity = 1;
    callable->closure = c;
    callable->function = p;
    trilogy_callable_init(t, callable);
}

void trilogy_callable_init_proc(
    trilogy_value* t, unsigned int arity, trilogy_array_value* c, void* p
) {
    trilogy_callable_value* callable =
        malloc_safe(sizeof(trilogy_callable_value));
    callable->tag = CALLABLE_PROCEDURE;
    callable->arity = arity;
    callable->closure = c;
    callable->function = p;
    trilogy_callable_init(t, callable);
}

void trilogy_callable_init_rule(
    trilogy_value* t, unsigned int arity, trilogy_array_value* c, void* p
) {
    trilogy_callable_value* callable =
        malloc_safe(sizeof(trilogy_callable_value));
    callable->tag = CALLABLE_RULE;
    callable->arity = arity;
    callable->closure = c;
    callable->function = p;
    trilogy_callable_init(t, callable);
}

void trilogy_callable_destroy(trilogy_callable_value* val) {
    if (val->closure != NO_CLOSURE) trilogy_array_destroy(val->closure);
}

trilogy_callable_value* trilogy_callable_untag(trilogy_value* val) {
    if (val->tag != TAG_CALLABLE) rte("callable", val->tag);
    return trilogy_callable_assume(val);
}

trilogy_callable_value* trilogy_callable_assume(trilogy_value* val) {
    return (void*)val->payload;
}

void* trilogy_function_untag(trilogy_callable_value* val) {
    if (val->tag != CALLABLE_FUNCTION)
        internal_panic("invalid application of non-function callable");
    return (void*)val->function;
}

void* trilogy_procedure_untag(trilogy_callable_value* val, unsigned int arity) {
    if (val->tag != CALLABLE_PROCEDURE)
        internal_panic("invalid call of non-procedure callable");
    if (val->arity != arity) internal_panic("procedure call arity mismatch");
    return (void*)val->function;
}

void* trilogy_rule_untag(trilogy_callable_value* val, unsigned int arity) {
    if (val->tag != CALLABLE_RULE)
        internal_panic("invalid call of non-rule callable");
    if (val->arity != arity) internal_panic("rule call arity mismatch");
    return (void*)val->function;
}
