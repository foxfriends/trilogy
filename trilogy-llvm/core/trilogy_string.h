#pragma once
#include "types.h"
#include <stdint.h>

trilogy_string_value*
trilogy_string_init(trilogy_value* tv, trilogy_string_value* str);
trilogy_string_value*
trilogy_string_init_new(trilogy_value* tv, uint64_t len, char* s);
trilogy_string_value*
trilogy_string_clone_into(trilogy_value* tv, const trilogy_string_value* orig);
trilogy_string_value* trilogy_string_init_from_c(trilogy_value* tv, char* s);

trilogy_string_value* trilogy_string_untag(trilogy_value* val);
trilogy_string_value* trilogy_string_assume(trilogy_value* val);

void trilogy_string_destroy(trilogy_string_value* val);

char* trilogy_string_as_c(trilogy_string_value* val);

uint64_t trilogy_string_len(trilogy_string_value* val);
uint32_t trilogy_string_at(trilogy_string_value* str, uint64_t index);
trilogy_string_value* trilogy_string_concat(
    trilogy_value* rt, trilogy_string_value* lhs, trilogy_string_value* rhs
);
int trilogy_string_compare(
    trilogy_string_value* lhs, trilogy_string_value* rhs
);
