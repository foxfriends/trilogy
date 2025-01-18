#pragma once
#include "types.h"

trilogy_value trilogy_tuple(trilogy_value* fst, trilogy_value* snd);
trilogy_value trilogy_tuple_clone(trilogy_tuple_value* tup);
trilogy_tuple_value* trilogy_tuple_untag(trilogy_value* val);
trilogy_tuple_value* trilogy_tuple_assume(trilogy_value* val);
void trilogy_tuple_destroy(trilogy_tuple_value* val);
