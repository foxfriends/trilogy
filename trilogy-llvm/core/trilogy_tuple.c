#include "trilogy_tuple.h"
#include "internal.h"
#include "trilogy_value.h"
#include <stdlib.h>

trilogy_tuple_value*
trilogy_tuple_init(trilogy_value* tv, trilogy_tuple_value* tup) {
    tv->tag = TAG_TUPLE;
    tv->payload = (unsigned long)tup;
    return tup;
}

trilogy_tuple_value* trilogy_tuple_init_new(
    trilogy_value* tv, trilogy_value* fst, trilogy_value* snd
) {
    trilogy_tuple_value* tup = malloc(sizeof(trilogy_tuple_value));
    tup->fst = *fst;
    tup->snd = *snd;
    return trilogy_tuple_init(tv, tup);
}

trilogy_tuple_value*
trilogy_tuple_clone_into(trilogy_value* tv, trilogy_tuple_value* orig) {
    trilogy_tuple_value* tup = malloc(sizeof(trilogy_tuple_value));
    trilogy_value_clone_into(&tup->fst, &orig->fst);
    trilogy_value_clone_into(&tup->snd, &orig->snd);
    return trilogy_tuple_init(tv, tup);
}

trilogy_tuple_value* trilogy_tuple_untag(trilogy_value* val) {
    if (val->tag != TAG_TUPLE) rte("tuple", val->tag);
    return trilogy_tuple_assume(val);
}

trilogy_tuple_value* trilogy_tuple_assume(trilogy_value* val) {
    return (trilogy_tuple_value*)val->payload;
}

void trilogy_tuple_destroy(trilogy_tuple_value* val) {
    trilogy_value_destroy(&val->fst);
    trilogy_value_destroy(&val->snd);
}
