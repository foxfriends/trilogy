#include <stdlib.h>
#include "trilogy_tuple.h"
#include "trilogy_value.h"
#include "internal.h"

trilogy_value trilogy_tuple(trilogy_value* fst, trilogy_value* snd) {
    trilogy_tuple_value* tup = malloc(sizeof(trilogy_tuple_value));
    tup->fst = *fst;
    tup->snd = *snd;
    trilogy_value t = { .tag = TAG_TUPLE, .payload = (unsigned long)tup };
    return t;
}

trilogy_value trilogy_tuple_clone(trilogy_tuple_value* orig) {
    trilogy_tuple_value* tup = malloc(sizeof(trilogy_tuple_value));
    tup->fst = trilogy_value_clone(&orig->fst);
    tup->snd = trilogy_value_clone(&orig->snd);
    trilogy_value t = { .tag = TAG_TUPLE, .payload = (unsigned long)tup };
    return t;
}

trilogy_tuple_value* untag_tuple(trilogy_value* val) {
    if (val->tag != TAG_TUPLE) rte("tuple", val->tag);
    return assume_tuple(val);
}

trilogy_tuple_value* assume_tuple(trilogy_value* val) {
    return (trilogy_tuple_value*)val->payload;
}

void trilogy_tuple_destroy(trilogy_tuple_value* val) {
    trilogy_value_destroy(&val->fst);
    trilogy_value_destroy(&val->snd);
}
