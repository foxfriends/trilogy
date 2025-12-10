#include "trilogy_callable.h"
#include "internal.h"
#include "trace.h"
#include "trilogy_array.h"
#include "trilogy_value.h"
#include "types.h"
#include <assert.h>
#include <stdint.h>
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
    trilogy_value* return_to, trilogy_value* yield_to, trilogy_value* closure,
    void* p
) {
    assert(
        closure == NO_CLOSURE || closure->tag == TAG_UNDEFINED ||
        closure->tag == TAG_ARRAY
    );
    callable->rc = 1;
    callable->tag = tag;
    callable->arity = arity;
    callable->return_to = NULL;
    if (return_to != NULL) {
        callable->return_to = trilogy_callable_assume(return_to);
        *return_to = trilogy_undefined;
    }
    callable->yield_to = NULL;
    if (yield_to != NULL) {
        callable->yield_to = trilogy_callable_assume(yield_to);
        *yield_to = trilogy_undefined;
    }
    callable->closure = NO_CLOSURE;
    if (closure != NO_CLOSURE && closure->tag != TAG_UNDEFINED) {
        callable->closure = trilogy_array_assume(closure);
        *closure = trilogy_undefined;
    }
    callable->function = p;
    TRACE("Initialized callable   (%d): %p\n", callable->tag, callable);
    return callable;
}

trilogy_callable_value* trilogy_callable_init_do(
    trilogy_value* t, uint32_t arity, trilogy_value* closure, void* p
) {
    trilogy_callable_value* callable =
        malloc_safe(sizeof(trilogy_callable_value));
    trilogy_callable_value_init(
        callable, CALLABLE_FUNCTION, arity, NULL, NULL, closure, p
    );
    return trilogy_callable_init(t, callable);
}

trilogy_callable_value* trilogy_callable_init_qy(
    trilogy_value* t, uint32_t arity, trilogy_value* closure, void* p
) {
    trilogy_callable_value* callable =
        malloc_safe(sizeof(trilogy_callable_value));
    trilogy_callable_value_init(
        callable, CALLABLE_RULE, arity, NULL, NULL, closure, p
    );
    return trilogy_callable_init(t, callable);
}

trilogy_callable_value*
trilogy_callable_init_proc(trilogy_value* t, uint32_t arity, void* p) {
    return trilogy_callable_init_do(t, arity, NO_CLOSURE, p);
}

trilogy_callable_value*
trilogy_callable_init_rule(trilogy_value* t, uint32_t arity, void* p) {
    return trilogy_callable_init_qy(t, arity, NO_CLOSURE, p);
}

trilogy_callable_value* trilogy_callable_init_cont(
    trilogy_value* t, trilogy_value* return_to, trilogy_value* yield_to,
    trilogy_value* closure, void* p
) {
    assert(
        closure == NO_CLOSURE || closure->tag == TAG_UNDEFINED ||
        closure->tag == TAG_ARRAY
    );
    trilogy_callable_value* callable =
        malloc_safe(sizeof(trilogy_callable_value));
    trilogy_callable_value_init(
        callable, CALLABLE_CONTINUATION, 1, return_to, yield_to, closure, p
    );
    return trilogy_callable_init(t, callable);
}

trilogy_callable_value* trilogy_callable_init_root(trilogy_value* t, void* p) {
    trilogy_callable_value* callable =
        malloc_safe(sizeof(trilogy_callable_value));
    // A weird little special case: the root level callbacks point to themself.
    // This is ok because we know the functions they point to will never use
    // these values, but we need SOME value in here so as not to act different
    // when being used.
    //
    // Don't increment the rc pointers extra though, this is accounted for
    // specifically in destroy.
    trilogy_callable_value_init(
        callable, CALLABLE_CONTINUATION, 1, NULL, NULL, NO_CLOSURE, p
    );
    callable->return_to = callable;
    callable->yield_to = callable;
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
        if (val->return_to != NULL && val->return_to != val) {
            trilogy_callable_destroy(val->return_to);
        }
        // Special case here, for the top level yield that points to itself
        // so that we don't double-destroy it.
        if (val->yield_to != NULL && val->yield_to != val) {
            trilogy_callable_destroy(val->yield_to);
        }
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

void trilogy_callable_promote(
    trilogy_value* tv, trilogy_value* return_to, trilogy_value* yield_to
) {
    trilogy_callable_value* original = trilogy_callable_untag(tv);
    trilogy_callable_value* clone = malloc_safe(sizeof(trilogy_callable_value));
    *clone = *original;
    clone->rc = 1;
    if (return_to != NULL) {
        clone->return_to = trilogy_callable_assume(return_to);
        *return_to = trilogy_undefined;
    } else if (clone->return_to != NULL) {
        clone->return_to->rc++;
    }
    if (yield_to != NULL) {
        clone->yield_to = trilogy_callable_assume(yield_to);
        *yield_to = trilogy_undefined;
    } else if (clone->yield_to != NULL) {
        clone->yield_to->rc++;
    }
    if (clone->closure != NO_CLOSURE) {
        clone->closure->rc++;
    }
    trilogy_value_destroy(tv);
    trilogy_callable_init(tv, clone);
}

trilogy_callable_value* trilogy_callable_untag(trilogy_value* val) {
    TRACE("Expect callable\t(%d): %p\n", val->tag, val);
    if (val->tag != TAG_CALLABLE) rte("callable", val->tag);
    return trilogy_callable_assume(val);
}

trilogy_callable_value* trilogy_callable_assume(trilogy_value* val) {
    assert(val->tag == TAG_CALLABLE);
    return (void*)val->payload;
}

void* trilogy_procedure_untag(trilogy_callable_value* val, uint32_t arity) {
    if (val->tag != CALLABLE_FUNCTION) {
        internal_panic("invalid call of non-procedure callable\n");
    }
    if (val->arity != arity) internal_panic("procedure call arity mismatch\n");
    return (void*)val->function;
}

void* trilogy_rule_untag(trilogy_callable_value* val, uint32_t arity) {
    if (val->tag != CALLABLE_RULE) {
        internal_panic("invalid call of non-rule callable\n");
    }
    if (val->arity != arity) internal_panic("rule call arity mismatch\n");
    return (void*)val->function;
}

void* trilogy_continuation_untag(trilogy_callable_value* val) {
    if (val->tag != CALLABLE_CONTINUATION) {
        internal_panic("invalid continue-to of non-continuation callable\n");
    }
    return (void*)val->function;
}
