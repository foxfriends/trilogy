#include "trilogy_number.h"
#include "internal.h"

trilogy_value trilogy_integer(long i) {
    trilogy_value t = { .tag = TAG_INTEGER, .payload = (unsigned long)i };
    return t;
}
long untag_integer(trilogy_value* val) {
    if (val->tag != TAG_INTEGER) rte("integer", val->tag);
    return assume_integer(val);
}
long assume_integer(trilogy_value* val) {
    return (long)val->payload;
}
