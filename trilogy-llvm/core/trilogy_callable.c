#include "trilogy_callable.h"
#include "internal.h"
#include "trilogy_value.h"
#include <assert.h>
#include <stdlib.h>

#define NO_CLOSURE 0

void trilogy_callable_init(trilogy_value* t, trilogy_callable_value* payload) {
    assert(t->tag == TAG_UNDEFINED);
    t->tag = TAG_CALLABLE;
    t->payload = (unsigned long)payload;
}

void trilogy_callable_clone_into(
    trilogy_value* t, trilogy_callable_value* orig
) {
    assert(orig->rc != 0);
    orig->rc++;
    trilogy_callable_init(t, orig);
}

void trilogy_callable_init_fn(
    trilogy_value* t, unsigned int closure_size, trilogy_value* c, void* p
) {
    trilogy_callable_value* callable =
        malloc_safe(sizeof(trilogy_callable_value));
    callable->rc = 1;
    callable->tag = CALLABLE_FUNCTION;
    callable->arity = 1;
    callable->closure_size = closure_size;
    callable->closure = c;
    callable->function = p;
    trilogy_callable_init(t, callable);
}

void trilogy_callable_init_do(
    trilogy_value* t, unsigned int arity, unsigned int closure_size,
    trilogy_value* c, void* p
) {
    trilogy_callable_value* callable =
        malloc_safe(sizeof(trilogy_callable_value));
    callable->rc = 1;
    callable->tag = CALLABLE_PROCEDURE;
    callable->arity = arity;
    callable->closure_size = closure_size;
    callable->closure = c;
    callable->function = p;
    trilogy_callable_init(t, callable);
}

void trilogy_callable_init_qy(
    trilogy_value* t, unsigned int arity, unsigned int closure_size,
    trilogy_value* c, void* p
) {
    trilogy_callable_value* callable =
        malloc_safe(sizeof(trilogy_callable_value));
    callable->rc = 1;
    callable->tag = CALLABLE_RULE;
    callable->arity = arity;
    callable->closure_size = closure_size;
    callable->closure = c;
    callable->function = p;
    trilogy_callable_init(t, callable);
}

void trilogy_callable_init_proc(trilogy_value* t, unsigned int arity, void* p) {
    trilogy_callable_init_do(t, arity, 0, NO_CLOSURE, p);
}

void trilogy_callable_init_func(trilogy_value* t, void* p) {
    trilogy_callable_init_fn(t, 0, NO_CLOSURE, p);
}

void trilogy_callable_init_rule(trilogy_value* t, unsigned int arity, void* p) {
    trilogy_callable_init_qy(t, arity, 0, NO_CLOSURE, p);
}

void trilogy_callable_destroy(trilogy_callable_value* val) {
    if (--val->rc == 0) {
        if (val->closure != NO_CLOSURE) {
            for (unsigned int i = 0; i < val->closure_size; ++i) {
                trilogy_value_destroy(&val->closure[i]);
            }
        }
        free(val);
    }
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
