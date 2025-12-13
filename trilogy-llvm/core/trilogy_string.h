#pragma once
#include "types.h"
#include <stddef.h>
#include <stdint.h>

trilogy_string_value*
trilogy_string_init(trilogy_value* tv, trilogy_string_value* str);
trilogy_string_value*
trilogy_string_init_new(trilogy_value* tv, size_t len, char* s);
trilogy_string_value*
trilogy_string_init_take(trilogy_value* tv, size_t len, char* s);
trilogy_string_value*
trilogy_string_clone_into(trilogy_value* tv, const trilogy_string_value* orig);
trilogy_string_value* trilogy_string_init_from_c(trilogy_value* tv, char* s);

trilogy_string_value* trilogy_string_untag(trilogy_value* val);
trilogy_string_value* trilogy_string_assume(trilogy_value* val);

void trilogy_string_destroy(trilogy_string_value* val);

char* trilogy_string_as_c(trilogy_string_value* val);

size_t trilogy_string_len(trilogy_string_value* val);
uint32_t trilogy_string_at(trilogy_string_value* str, size_t index);
trilogy_string_value* trilogy_string_concat(
    trilogy_value* rt, trilogy_string_value* lhs, trilogy_string_value* rhs
);
int trilogy_string_compare(
    trilogy_string_value* lhs, trilogy_string_value* rhs
);
void trilogy_string_slice(
    trilogy_value* tv, trilogy_string_value* str, size_t start, size_t end
);

bool trilogy_string_unglue_start(
    trilogy_value* rt, trilogy_string_value* lhs, trilogy_string_value* rhs
);
bool trilogy_string_unglue_end(
    trilogy_value* rt, trilogy_string_value* lhs, trilogy_string_value* rhs
);

void trilogy_string_to_array(trilogy_value* rt, trilogy_string_value* str);
