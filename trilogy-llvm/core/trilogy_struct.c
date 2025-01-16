#include <stdlib.h>
#include "trilogy_struct.h"
#include "trilogy_value.h"
#include "internal.h"

trilogy_value trilogy_struct(unsigned long i, trilogy_value* val) {
    trilogy_struct_value* st = malloc(sizeof(trilogy_struct_value));
    st->atom = i;
    st->contents = val;
    trilogy_value t = { .tag = TAG_STRUCT, .payload = (unsigned long)st };
    return t;
}

trilogy_struct_value* untag_struct(trilogy_value* val) {
    if (val->tag != TAG_STRUCT) rte("struct", val->tag);
    return assume_struct(val);
}

trilogy_struct_value* assume_struct(trilogy_value* val) {
    return (trilogy_struct_value*)val->payload;
}

void destroy_struct(trilogy_struct_value* val) {
    destroy_trilogy_value(val->contents);
    free(val->contents);
}
