#include "trilogy_boolean.h"
#include "internal.h"
#include <assert.h>
#include <stdint.h>

const trilogy_value trilogy_true = {.tag = TAG_BOOL, .payload = 0};

const trilogy_value trilogy_false = {.tag = TAG_BOOL, .payload = 1};

void trilogy_boolean_init(trilogy_value* t, bool b) {
    assert(t->tag == TAG_UNDEFINED);
    t->tag = TAG_BOOL;
    t->payload = (uint64_t)b;
}

bool trilogy_boolean_untag(trilogy_value* val) {
    if (val->tag != TAG_BOOL) rte("boolean", val->tag);
    return trilogy_boolean_assume(val);
}

bool trilogy_boolean_assume(trilogy_value* val) {
    assert(val->tag == TAG_BOOL);
    return (bool)val->payload;
}

int trilogy_boolean_compare(bool lhs, bool rhs) {
    return ((int)lhs - (int)rhs);
}

void trilogy_boolean_not(trilogy_value* rv, trilogy_value* v) {
    trilogy_boolean_init(rv, !trilogy_boolean_untag(v));
}
