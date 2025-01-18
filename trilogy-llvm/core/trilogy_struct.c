#include <stdlib.h>
#include "trilogy_struct.h"
#include "trilogy_value.h"
#include "internal.h"

trilogy_value trilogy_struct_new(unsigned long i, trilogy_value* val) {
    trilogy_struct_value* st = malloc(sizeof(trilogy_struct_value));
    st->atom = i;
    st->contents = *val;
    trilogy_value t = { .tag = TAG_STRUCT, .payload = (unsigned long)st };
    return t;
}

trilogy_value trilogy_struct_clone(trilogy_struct_value* val) {
    trilogy_struct_value* st = malloc(sizeof(trilogy_struct_value));
    st->atom = val->atom;
    st->contents = trilogy_value_clone(&val->contents);
    trilogy_value t = { .tag = TAG_STRUCT, .payload = (unsigned long)st };
    return t;
}

trilogy_struct_value* trilogy_struct_untag(trilogy_value* val) {
    if (val->tag != TAG_STRUCT) rte("struct", val->tag);
    return trilogy_struct_assume(val);
}

trilogy_struct_value* trilogy_struct_assume(trilogy_value* val) {
    return (trilogy_struct_value*)val->payload;
}

void trilogy_struct_destroy(trilogy_struct_value* val) {
    trilogy_value_destroy(&val->contents);
}
