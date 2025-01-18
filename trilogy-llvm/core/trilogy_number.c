#include "trilogy_number.h"
#include "internal.h"

trilogy_value trilogy_integer(long i) {
    trilogy_value t = { .tag = TAG_INTEGER, .payload = (unsigned long)i };
    return t;
}
long trilogy_integer_untag(trilogy_value* val) {
    if (val->tag != TAG_INTEGER) rte("integer", val->tag);
    return trilogy_integer_assume(val);
}
long trilogy_integer_assume(trilogy_value* val) {
    return (long)val->payload;
}
