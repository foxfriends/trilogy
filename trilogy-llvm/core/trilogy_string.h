#pragma once
#include "types.h"

trilogy_string_value*
trilogy_string_init(trilogy_value* tv, trilogy_string_value* str);
trilogy_string_value*
trilogy_string_init_new(trilogy_value* tv, unsigned long len, char* s);
trilogy_string_value*
trilogy_string_clone_into(trilogy_value* tv, trilogy_string_value* orig);
trilogy_string_value* trilogy_string_init_from_c(trilogy_value* tv, char* s);

trilogy_string_value* trilogy_string_untag(trilogy_value* val);
trilogy_string_value* trilogy_string_assume(trilogy_value* val);

void trilogy_string_destroy(trilogy_string_value* val);

char* trilogy_string_as_c(trilogy_string_value* val);
unsigned long trilogy_string_len(trilogy_string_value* val);

void trilogy_string_glue(
    trilogy_value* rt, trilogy_value* lhs, trilogy_value* rhs
);
