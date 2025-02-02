#pragma once
#include "types.h"

trilogy_tuple_value*
trilogy_tuple_init(trilogy_value* tv, trilogy_tuple_value* tup);
trilogy_tuple_value* trilogy_tuple_init_new(
    trilogy_value* tv, trilogy_value* fst, trilogy_value* snd
);
trilogy_tuple_value*
trilogy_tuple_clone_into(trilogy_value* tv, trilogy_tuple_value* tup);

trilogy_tuple_value* trilogy_tuple_untag(trilogy_value* val);
trilogy_tuple_value* trilogy_tuple_assume(trilogy_value* val);

void trilogy_tuple_left(trilogy_value* val, trilogy_tuple_value* tup);
void trilogy_tuple_right(trilogy_value* val, trilogy_tuple_value* tup);

void trilogy_tuple_destroy(trilogy_tuple_value* val);
