#include "trilogy_number.h"
#include "internal.h"

trilogy_value trilogy_number(long i) {
    trilogy_value t = {.tag = TAG_NUMBER, .payload = (unsigned long)i};
    return t;
}
long trilogy_number_untag(trilogy_value* val) {
    if (val->tag != TAG_NUMBER) rte("number", val->tag);
    return trilogy_number_assume(val);
}
long trilogy_number_assume(trilogy_value* val) { return (long)val->payload; }
