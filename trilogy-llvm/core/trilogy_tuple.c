#include "trilogy_tuple.h"
#include "internal.h"
#include "trilogy_value.h"
#include "types.h"
#include <assert.h>
#include <stdint.h>

trilogy_tuple_value*
trilogy_tuple_init(trilogy_value* tv, trilogy_tuple_value* tup) {
    assert(tv->tag == TAG_UNDEFINED);
    tv->tag = TAG_TUPLE;
    tv->payload = (uint64_t)tup;
    return tup;
}

trilogy_tuple_value* trilogy_tuple_init_new(
    trilogy_value* tv, trilogy_value* fst, trilogy_value* snd
) {
    trilogy_tuple_value* tup = malloc_safe(sizeof(trilogy_tuple_value));
    tup->fst = *fst;
    tup->snd = *snd;
    return trilogy_tuple_init(tv, tup);
}

trilogy_tuple_value*
trilogy_tuple_clone_into(trilogy_value* tv, trilogy_tuple_value* orig) {
    trilogy_tuple_value* tup = malloc_safe(sizeof(trilogy_tuple_value));
    tup->fst = trilogy_undefined;
    tup->snd = trilogy_undefined;
    trilogy_value_clone_into(&tup->fst, &orig->fst);
    trilogy_value_clone_into(&tup->snd, &orig->snd);
    return trilogy_tuple_init(tv, tup);
}

trilogy_tuple_value* trilogy_tuple_untag(trilogy_value* val) {
    if (val->tag != TAG_TUPLE) rte("tuple", val->tag);
    return trilogy_tuple_assume(val);
}

trilogy_tuple_value* trilogy_tuple_assume(trilogy_value* val) {
    assert(val->tag == TAG_TUPLE);
    return (trilogy_tuple_value*)val->payload;
}

void trilogy_tuple_left(trilogy_value* val, trilogy_tuple_value* tup) {
    trilogy_value_clone_into(val, &tup->fst);
}

void trilogy_tuple_right(trilogy_value* val, trilogy_tuple_value* tup) {
    trilogy_value_clone_into(val, &tup->snd);
}

int trilogy_tuple_compare(trilogy_tuple_value* lhs, trilogy_tuple_value* rhs) {
    int cmp = trilogy_value_compare(&lhs->fst, &rhs->fst);
    if (cmp != 0) return cmp;
    return trilogy_value_compare(&lhs->snd, &rhs->snd);
}

void trilogy_tuple_destroy(trilogy_tuple_value* val) {
    trilogy_value_destroy(&val->fst);
    trilogy_value_destroy(&val->snd);
}
