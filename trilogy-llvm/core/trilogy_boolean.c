#include "trilogy_boolean.h"
#include "internal.h"

const trilogy_value trilogy_true = { .tag = TAG_BOOL, .payload = 0 };

const trilogy_value trilogy_false = { .tag = TAG_BOOL, .payload = 1 };

trilogy_value trilogy_bool(bool b) {
    trilogy_value t = { .tag = TAG_BOOL, .payload = b };
    return t;
}

bool untag_bool(trilogy_value* val) {
    if (val->tag != TAG_BOOL) rte("bool", val->tag);
    return assume_bool(val);
}

bool assume_bool(trilogy_value* val) {
    return (bool)val->payload;
}
