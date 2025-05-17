#pragma once
#include "types.h"
#include <stddef.h>

trilogy_array_value*
trilogy_array_init(trilogy_value* tv, trilogy_array_value* arr);

trilogy_array_value* trilogy_array_init_empty(trilogy_value* tv);

trilogy_array_value* trilogy_array_init_cap(trilogy_value* tv, size_t cap);

trilogy_array_value*
trilogy_array_clone_into(trilogy_value* tv, trilogy_array_value* arr);

size_t trilogy_array_len(trilogy_array_value* tv);
size_t trilogy_array_cap(trilogy_array_value* tv);
size_t trilogy_array_resize(trilogy_array_value* tv, size_t cap);
size_t trilogy_array_reserve(trilogy_array_value* tv, size_t cap);

void trilogy_array_push(trilogy_array_value* arr, trilogy_value* tv);
void trilogy_array_set(
    trilogy_array_value* arr, size_t index, trilogy_value* tv
);
void trilogy_array_append(trilogy_array_value* arr, trilogy_value* tv);
void trilogy_array_at(
    trilogy_value* tv, trilogy_array_value* arr, size_t index
);
void trilogy_array_slice(
    trilogy_value* tv, trilogy_array_value* arr, size_t start, size_t end
);

int trilogy_array_compare(trilogy_array_value* lhs, trilogy_array_value* rhs);

trilogy_array_value* trilogy_array_untag(trilogy_value* val);
trilogy_array_value* trilogy_array_assume(trilogy_value* val);

void trilogy_array_destroy(trilogy_array_value* arr);
