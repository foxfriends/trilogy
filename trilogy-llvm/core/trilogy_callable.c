#include "trilogy_callable.h"
#include "internal.h"
#include "trace.h"
#include "trilogy_array.h"
#include "trilogy_value.h"
#include <assert.h>
#include <stdlib.h>

trilogy_value* prepare_closure(uint32_t closure_size) {
    return calloc_safe(closure_size, sizeof(trilogy_value));
}

trilogy_callable_value*
trilogy_callable_init(trilogy_value* t, trilogy_callable_value* payload) {
    assert(t->tag == TAG_UNDEFINED);
    t->tag = TAG_CALLABLE;
    t->payload = (uint64_t)payload;
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

static trilogy_callable_value* trilogy_callable_value_init(
    trilogy_callable_value* callable, trilogy_callable_tag tag, uint32_t arity,
    trilogy_value* return_to, trilogy_value* yield_to, trilogy_value* cancel_to,
    trilogy_value* break_to, trilogy_value* continue_to, trilogy_value* closure,
    void* p
) {
    assert(closure == NO_CLOSURE || closure->tag == TAG_ARRAY);
    callable->rc = 1;
    callable->tag = tag;
    callable->arity = arity;
    callable->return_to =
        return_to == NULL ? NULL : trilogy_callable_assume(return_to);
    callable->yield_to =
        yield_to == NULL ? NULL : trilogy_callable_assume(yield_to);
    callable->cancel_to =
        cancel_to == NULL ? NULL : trilogy_callable_assume(cancel_to);
    callable->break_to =
        break_to == NULL ? NULL : trilogy_callable_assume(break_to);
    callable->continue_to =
        continue_to == NULL ? NULL : trilogy_callable_assume(continue_to);
    callable->closure =
        closure == NO_CLOSURE ? NO_CLOSURE : trilogy_array_assume(closure);
    callable->function = p;
    TRACE("Initialized callable   (%d): %p\n", callable->tag, callable);
}

trilogy_callable_value*
trilogy_callable_init_fn(trilogy_value* t, trilogy_value* closure, void* p) {
    trilogy_callable_value* callable =
        malloc_safe(sizeof(trilogy_callable_value));
    trilogy_callable_value_init(
        callable, CALLABLE_FUNCTION, 1, NULL, NULL, NULL, NULL, NULL, closure, p
    );
    return trilogy_callable_init(t, callable);
}

trilogy_callable_value* trilogy_callable_init_do(
    trilogy_value* t, uint32_t arity, trilogy_value* closure, void* p
) {
    trilogy_callable_value* callable =
        malloc_safe(sizeof(trilogy_callable_value));
    trilogy_callable_value_init(
        callable, CALLABLE_PROCEDURE, arity, NULL, NULL, NULL, NULL, NULL,
        closure, p
    );
    return trilogy_callable_init(t, callable);
}

trilogy_callable_value* trilogy_callable_init_qy(
    trilogy_value* t, uint32_t arity, trilogy_value* closure, void* p
) {
    trilogy_callable_value* callable =
        malloc_safe(sizeof(trilogy_callable_value));
    trilogy_callable_value_init(
        callable, CALLABLE_RULE, arity, NULL, NULL, NULL, NULL, NULL, closure, p
    );
    return trilogy_callable_init(t, callable);
}

trilogy_callable_value*
trilogy_callable_init_proc(trilogy_value* t, uint32_t arity, void* p) {
    return trilogy_callable_init_do(t, arity, NO_CLOSURE, p);
}

trilogy_callable_value* trilogy_callable_init_func(trilogy_value* t, void* p) {
    return trilogy_callable_init_fn(t, NO_CLOSURE, p);
}

trilogy_callable_value*
trilogy_callable_init_rule(trilogy_value* t, uint32_t arity, void* p) {
    return trilogy_callable_init_qy(t, arity, NO_CLOSURE, p);
}

trilogy_callable_value* trilogy_callable_init_cont(
    trilogy_value* t, trilogy_value* return_to, trilogy_value* yield_to,
    trilogy_value* cancel_to, trilogy_value* break_to,
    trilogy_value* continue_to, trilogy_value* closure, void* p
) {
    assert(closure != NO_CLOSURE);
    assert(closure->tag == TAG_ARRAY);
    trilogy_callable_value* callable =
        malloc_safe(sizeof(trilogy_callable_value));
    trilogy_callable_value_init(
        callable, CALLABLE_CONTINUATION, 1, return_to, yield_to, cancel_to,
        break_to, continue_to, closure, p
    );
    return trilogy_callable_init(t, callable);
}

trilogy_callable_value* trilogy_callable_init_resume(
    trilogy_value* t, trilogy_value* return_to, trilogy_value* yield_to,
    trilogy_value* cancel_to, trilogy_value* break_to,
    trilogy_value* continue_to, trilogy_value* closure, void* p
) {
    assert(closure != NO_CLOSURE);
    assert(closure->tag == TAG_ARRAY);
    trilogy_callable_value* callable =
        malloc_safe(sizeof(trilogy_callable_value));
    trilogy_callable_value_init(
        callable, CALLABLE_RESUME, 1, return_to, yield_to, cancel_to, break_to,
        continue_to, closure, p
    );
    return trilogy_callable_init(t, callable);
}

trilogy_callable_value* trilogy_callable_init_continue(
    trilogy_value* t, trilogy_value* return_to, trilogy_value* yield_to,
    trilogy_value* cancel_to, trilogy_value* break_to,
    trilogy_value* continue_to, trilogy_value* closure, void* p
) {
    assert(closure != NO_CLOSURE);
    assert(closure->tag == TAG_ARRAY);
    trilogy_callable_value* callable =
        malloc_safe(sizeof(trilogy_callable_value));
    trilogy_callable_value_init(
        callable, CALLABLE_CONTINUE, 1, return_to, yield_to, cancel_to,
        break_to, continue_to, closure, p
    );
    return trilogy_callable_init(t, callable);
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
        if (val->break_to != NULL) trilogy_callable_destroy(val->break_to);
        if (val->continue_to != NULL)
            trilogy_callable_destroy(val->continue_to);
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

void trilogy_callable_break_to_into(
    trilogy_value* val, trilogy_callable_value* cal
) {
    if (cal->break_to == NULL) return;
    trilogy_callable_clone_into(val, cal->break_to);
}

void trilogy_callable_continue_to_into(
    trilogy_value* val, trilogy_callable_value* cal
) {
    if (cal->continue_to == NULL) return;
    trilogy_callable_clone_into(val, cal->continue_to);
}

static void shift(
    trilogy_value* val, trilogy_value* cancel_to, trilogy_callable_value* cal
) {
    trilogy_value return_to = trilogy_undefined;
    trilogy_value yield_to = trilogy_undefined;
    trilogy_value break_to = trilogy_undefined;
    trilogy_value continue_to = trilogy_undefined;
    trilogy_value closure = trilogy_undefined;
    trilogy_callable_return_to_into(&return_to, cal);
    trilogy_callable_yield_to_into(&yield_to, cal);
    trilogy_callable_break_to_into(&break_to, cal);
    trilogy_callable_continue_to_into(&continue_to, cal);
    trilogy_callable_closure_into(&closure, cal);
    trilogy_callable_init_cont(
        val, &return_to, &yield_to, cancel_to, &break_to, &continue_to,
        &closure, cal->function
    );
}

void trilogy_callable_yield_to_shift(
    trilogy_value* val, trilogy_value* cancel_to, trilogy_callable_value* cal
) {
    assert(cal->yield_to != NULL);
    shift(val, cancel_to, cal->yield_to);
    *cancel_to = trilogy_undefined;
}

void trilogy_callable_return_to_shift(
    trilogy_value* val, trilogy_value* cancel_to, trilogy_callable_value* cal
) {
    assert(cal->return_to != NULL);
    shift(val, cancel_to, cal->return_to);
    *cancel_to = trilogy_undefined;
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

void* trilogy_procedure_untag(trilogy_callable_value* val, uint32_t arity) {
    if (val->tag != CALLABLE_PROCEDURE)
        internal_panic("invalid call of non-procedure callable\n");
    if (val->arity != arity) internal_panic("procedure call arity mismatch\n");
    return (void*)val->function;
}

void* trilogy_rule_untag(trilogy_callable_value* val, uint32_t arity) {
    if (val->tag != CALLABLE_RULE)
        internal_panic("invalid call of non-rule callable\n");
    if (val->arity != arity) internal_panic("rule call arity mismatch\n");
    return (void*)val->function;
}

void* trilogy_continuation_untag(trilogy_callable_value* val) {
    if (val->tag != CALLABLE_CONTINUATION && val->tag != CALLABLE_RESUME &&
        val->tag != CALLABLE_CONTINUE)
        internal_panic("invalid continue-to of non-continuation callable\n");
    return (void*)val->function;
}
