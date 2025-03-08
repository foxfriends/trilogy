#include "trilogy_callable.h"
#include "internal.h"
#include "trace.h"
#include "trilogy_array.h"
#include "trilogy_value.h"
#include <assert.h>
#include <stdlib.h>

trilogy_value* prepare_closure(unsigned int closure_size) {
    return calloc_safe(closure_size, sizeof(trilogy_value));
}

trilogy_callable_value*
trilogy_callable_init(trilogy_value* t, trilogy_callable_value* payload) {
    assert(t->tag == TAG_UNDEFINED);
    t->tag = TAG_CALLABLE;
    t->payload = (unsigned long)payload;
    return payload;
}

void trilogy_callable_clone_into(
    trilogy_value* t, trilogy_callable_value* orig
) {
    assert(orig->rc != 0);
    TRACE(
        "Cloning callable    (%d): %p (%lu -> %lu)\n", orig->tag, orig,
        orig->rc, orig->rc + 1
    );
    orig->rc++;
    trilogy_callable_init(t, orig);
}

trilogy_callable_value*
trilogy_callable_init_fn(trilogy_value* t, trilogy_value* closure, void* p) {
    assert(closure == NO_CLOSURE || closure->tag == TAG_ARRAY);
    trilogy_callable_value* callable =
        malloc_safe(sizeof(trilogy_callable_value));
    callable->rc = 1;
    callable->tag = CALLABLE_FUNCTION;
    callable->arity = 1;
    callable->return_to = NULL;
    callable->yield_to = NULL;
    callable->cancel_to = NULL;
    callable->closure =
        closure == NO_CLOSURE ? NO_CLOSURE : trilogy_array_assume(closure);
    callable->function = p;
    TRACE("Creating callable   (%d): %p\n", callable->tag, callable);
    return trilogy_callable_init(t, callable);
}

trilogy_callable_value* trilogy_callable_init_do(
    trilogy_value* t, unsigned int arity, trilogy_value* closure, void* p
) {
    assert(closure == NO_CLOSURE || closure->tag == TAG_ARRAY);
    trilogy_callable_value* callable =
        malloc_safe(sizeof(trilogy_callable_value));
    callable->rc = 1;
    callable->tag = CALLABLE_PROCEDURE;
    callable->arity = arity;
    callable->return_to = NULL;
    callable->yield_to = NULL;
    callable->cancel_to = NULL;
    callable->closure =
        closure == NO_CLOSURE ? NO_CLOSURE : trilogy_array_assume(closure);
    callable->function = p;
    TRACE("Creating callable   (%d): %p\n", callable->tag, callable);
    return trilogy_callable_init(t, callable);
}

trilogy_callable_value* trilogy_callable_init_qy(
    trilogy_value* t, unsigned int arity, trilogy_value* closure, void* p
) {
    assert(closure == NO_CLOSURE || closure->tag == TAG_ARRAY);
    trilogy_callable_value* callable =
        malloc_safe(sizeof(trilogy_callable_value));
    callable->rc = 1;
    callable->tag = CALLABLE_RULE;
    callable->arity = arity;
    callable->return_to = NULL;
    callable->yield_to = NULL;
    callable->cancel_to = NULL;
    callable->closure =
        closure == NO_CLOSURE ? NO_CLOSURE : trilogy_array_assume(closure);
    callable->function = p;
    TRACE("Creating callable   (%d): %p\n", callable->tag, callable);
    return trilogy_callable_init(t, callable);
}

trilogy_callable_value*
trilogy_callable_init_proc(trilogy_value* t, unsigned int arity, void* p) {
    return trilogy_callable_init_do(t, arity, NO_CLOSURE, p);
}

trilogy_callable_value* trilogy_callable_init_func(trilogy_value* t, void* p) {
    return trilogy_callable_init_fn(t, NO_CLOSURE, p);
}

trilogy_callable_value*
trilogy_callable_init_rule(trilogy_value* t, unsigned int arity, void* p) {
    return trilogy_callable_init_qy(t, arity, NO_CLOSURE, p);
}

trilogy_callable_value* trilogy_callable_init_cont(
    trilogy_value* t, trilogy_value* return_to, trilogy_value* yield_to,
    trilogy_value* cancel_to, trilogy_value* closure, void* p
) {
    assert(closure != NO_CLOSURE);
    assert(closure->tag == TAG_ARRAY);
    trilogy_callable_value* callable =
        malloc_safe(sizeof(trilogy_callable_value));
    callable->rc = 1;
    callable->tag = CALLABLE_CONTINUATION;
    callable->arity = 1;
    callable->return_to =
        return_to == NULL ? NULL : trilogy_callable_assume(return_to);
    callable->yield_to =
        yield_to == NULL ? NULL : trilogy_callable_assume(yield_to);
    callable->cancel_to =
        cancel_to == NULL ? NULL : trilogy_callable_assume(cancel_to);
    callable->closure =
        closure == NO_CLOSURE ? NO_CLOSURE : trilogy_array_assume(closure);
    callable->function = p;
    TRACE("Creating callable   (%d): %p\n", callable->tag, callable);
    return trilogy_callable_init(t, callable);
}

trilogy_callable_value* trilogy_callable_init_resume(
    trilogy_value* t, trilogy_value* return_to, trilogy_value* yield_to,
    trilogy_value* cancel_to, trilogy_value* closure, void* p
) {
    trilogy_callable_value* callable = trilogy_callable_init_cont(
        t, return_to, yield_to, cancel_to, closure, p
    );
    callable->tag = CALLABLE_RESUME;
    return callable;
}

void trilogy_callable_destroy(trilogy_callable_value* val) {
    assert(val->rc > 0);
    TRACE(
        "Destroying callable (%d): %p (%lu -> %lu)\n", val->tag, val, val->rc,
        val->rc - 1
    );
    if (--val->rc == 0) {
        TRACE("\tDeallocating!\n");
        if (val->closure != NO_CLOSURE) trilogy_array_destroy(val->closure);
        // NOTE: even a continuation may have return_to and yield_to as NULL, as
        // is the case in the wrapper of main.
        if (val->return_to != NULL) trilogy_callable_destroy(val->return_to);
        if (val->yield_to != NULL) trilogy_callable_destroy(val->yield_to);
        if (val->cancel_to != NULL) trilogy_callable_destroy(val->cancel_to);
        free(val);
        TRACE("\tDeallocated!\n");
    }
}

trilogy_array_value*
trilogy_callable_closure_into(trilogy_value* val, trilogy_callable_value* cal) {
    assert(val->tag == TAG_UNDEFINED);
    if (cal->closure == NO_CLOSURE) return NULL;
    return trilogy_array_clone_into(val, cal->closure);
}

void trilogy_callable_return_to_into(
    trilogy_value* val, trilogy_callable_value* cal
) {
    if (cal->return_to == NULL) return;
    trilogy_callable_clone_into(val, cal->return_to);
}

void trilogy_callable_yield_to_into(
    trilogy_value* val, trilogy_callable_value* cal
) {
    if (cal->yield_to == NULL) return;
    trilogy_callable_clone_into(val, cal->yield_to);
}

void trilogy_callable_cancel_to_into(
    trilogy_value* val, trilogy_callable_value* cal
) {
    if (cal->cancel_to == NULL) return;
    trilogy_callable_clone_into(val, cal->cancel_to);
}

void trilogy_callable_yield_to_shift(
    trilogy_value* val, trilogy_value* cancel_to, trilogy_callable_value* cal
) {
    assert(cal->yield_to != NULL);
    trilogy_value return_to = trilogy_undefined;
    trilogy_value yield_to = trilogy_undefined;
    trilogy_value closure = trilogy_undefined;
    trilogy_callable_return_to_into(&return_to, cal->yield_to);
    trilogy_callable_yield_to_into(&yield_to, cal->yield_to);
    trilogy_callable_closure_into(&closure, cal->yield_to);
    trilogy_callable_init_cont(
        val, &return_to, &yield_to, cancel_to, &closure, cal->yield_to->function
    );
}

void trilogy_callable_return_to_shift(
    trilogy_value* val, trilogy_value* cancel_to, trilogy_callable_value* cal
) {
    assert(cal->return_to != NULL);
    trilogy_value return_to = trilogy_undefined;
    trilogy_value yield_to = trilogy_undefined;
    trilogy_value closure = trilogy_undefined;
    trilogy_callable_return_to_into(&return_to, cal->return_to);
    trilogy_callable_yield_to_into(&yield_to, cal->return_to);
    trilogy_callable_closure_into(&closure, cal->return_to);
    trilogy_callable_init_cont(
        val, &return_to, &yield_to, cancel_to, &closure,
        cal->return_to->function
    );
}

trilogy_callable_value* trilogy_callable_untag(trilogy_value* val) {
    TRACE("Expect callable: %p\n", val);
    if (val->tag != TAG_CALLABLE) rte("callable", val->tag);
    return trilogy_callable_assume(val);
}

trilogy_callable_value* trilogy_callable_assume(trilogy_value* val) {
    assert(val->tag == TAG_CALLABLE);
    return (void*)val->payload;
}

void* trilogy_function_untag(trilogy_callable_value* val) {
    if (val->tag != CALLABLE_FUNCTION)
        internal_panic("invalid application of non-function callable\n");
    return (void*)val->function;
}

void* trilogy_procedure_untag(trilogy_callable_value* val, unsigned int arity) {
    if (val->tag != CALLABLE_PROCEDURE)
        internal_panic("invalid call of non-procedure callable\n");
    if (val->arity != arity) internal_panic("procedure call arity mismatch\n");
    return (void*)val->function;
}

void* trilogy_rule_untag(trilogy_callable_value* val, unsigned int arity) {
    if (val->tag != CALLABLE_RULE)
        internal_panic("invalid call of non-rule callable\n");
    if (val->arity != arity) internal_panic("rule call arity mismatch\n");
    return (void*)val->function;
}

void* trilogy_continuation_untag(trilogy_callable_value* val) {
    if (val->tag != CALLABLE_CONTINUATION && val->tag != CALLABLE_RESUME)
        internal_panic("invalid continue-to of non-continuation callable\n");
    return (void*)val->function;
}

bool trilogy_continuation_is_resume(trilogy_callable_value* val) {
    return val->tag == CALLABLE_RESUME;
}
