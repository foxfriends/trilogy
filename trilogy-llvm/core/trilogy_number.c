#include "trilogy_number.h"
#include "internal.h"
#include <assert.h>

void trilogy_number_init(trilogy_value* tv, long n) {
    assert(tv->tag == TAG_UNDEFINED);
    tv->tag = TAG_NUMBER;
    tv->payload = (unsigned long)n;
}

long trilogy_number_untag(trilogy_value* val) {
    if (val->tag != TAG_NUMBER) rte("number", val->tag);
    return trilogy_number_assume(val);
}

long trilogy_number_assume(trilogy_value* val) {
    assert(val->tag == TAG_NUMBER);
    return (long)val->payload;
}
