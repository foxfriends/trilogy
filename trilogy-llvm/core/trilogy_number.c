#include "trilogy_number.h"
#include "internal.h"
#include <assert.h>

void trilogy_number_init(trilogy_value* tv, trilogy_number_value n) {
    assert(tv->tag == TAG_UNDEFINED);
    tv->tag = TAG_NUMBER;
    tv->payload = (unsigned long)n;
}

trilogy_number_value trilogy_number_untag(trilogy_value* val) {
    if (val->tag != TAG_NUMBER) rte("number", val->tag);
    return trilogy_number_assume(val);
}

trilogy_number_value trilogy_number_assume(trilogy_value* val) {
    assert(val->tag == TAG_NUMBER);
    return (trilogy_number_value)val->payload;
}

int trilogy_number_compare(trilogy_number_value lhs, trilogy_number_value rhs) {
    if (lhs < rhs) return -1;
    if (lhs > rhs) return 1;
    return 0;
}
