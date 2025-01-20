#include "trilogy_number.h"
#include "internal.h"

void trilogy_number_init(trilogy_value* tv, long n) {
    tv->tag = TAG_NUMBER;
    tv->payload = (unsigned long)n;
}

long trilogy_number_untag(trilogy_value* val) {
    if (val->tag != TAG_NUMBER) rte("number", val->tag);
    return trilogy_number_assume(val);
}

long trilogy_number_assume(trilogy_value* val) { return (long)val->payload; }
