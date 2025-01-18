#include <stdlib.h>
#include "trilogy_struct.h"
#include "trilogy_value.h"
#include "internal.h"

trilogy_struct_value* trilogy_struct_init(trilogy_value* tv, trilogy_struct_value* st) {
    tv->tag = TAG_STRUCT;
    tv->payload = (unsigned long)st;
    return st;
}

trilogy_struct_value* trilogy_struct_new(trilogy_value* tv, unsigned long i, trilogy_value* val) {
    trilogy_struct_value* st = malloc(sizeof(trilogy_struct_value));
    st->atom = i;
    st->contents = *val;
    return trilogy_struct_init(tv, st);
}

trilogy_struct_value* trilogy_struct_clone_into(trilogy_value* tv, trilogy_struct_value* val) {
    trilogy_struct_value* st = malloc(sizeof(trilogy_struct_value));
    st->atom = val->atom;
    trilogy_value_clone_into(&st->contents, &val->contents);
    return trilogy_struct_init(tv, st);
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
