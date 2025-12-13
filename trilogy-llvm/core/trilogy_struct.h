#pragma once
#include "types.h"
#include <stdint.h>

trilogy_struct_value*
trilogy_struct_init(trilogy_value* tv, trilogy_struct_value* st);
trilogy_struct_value*
trilogy_struct_init_new(trilogy_value* tv, uint64_t i, trilogy_value* val);
trilogy_struct_value*
trilogy_struct_init_take(trilogy_value* tv, uint64_t i, trilogy_value* val);
trilogy_struct_value*
trilogy_struct_clone_into(trilogy_value* tv, trilogy_struct_value* val);

trilogy_struct_value* trilogy_struct_untag(trilogy_value* val);
trilogy_struct_value* trilogy_struct_assume(trilogy_value* val);

void trilogy_struct_destroy(trilogy_struct_value* val);

int trilogy_struct_compare(
    trilogy_struct_value* lhs, trilogy_struct_value* rhs
);
