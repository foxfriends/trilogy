#pragma once
#include "types.h"
#include <stdbool.h>

trilogy_set_value* trilogy_set_init(trilogy_value* tv, trilogy_set_value* set);
trilogy_set_value* trilogy_set_init_empty(trilogy_value* tv);
trilogy_set_value* trilogy_set_init_cap(trilogy_value* tv, size_t cap);
trilogy_set_value*
trilogy_set_clone_into(trilogy_value* tv, trilogy_set_value* arr);
trilogy_set_value*
trilogy_set_deep_clone_into(trilogy_value* tv, trilogy_set_value* rec);

size_t trilogy_set_len(trilogy_set_value* tv);
size_t trilogy_set_cap(trilogy_set_value* tv);

void trilogy_set_insert(trilogy_set_value* set, trilogy_value* value);
void trilogy_set_append(trilogy_set_value* set, trilogy_value* value);
bool trilogy_set_delete(trilogy_set_value* set, trilogy_value* value);
bool trilogy_set_contains(trilogy_set_value* set, trilogy_value* key);

bool trilogy_set_structural_eq(trilogy_set_value* lhs, trilogy_set_value* rhs);

trilogy_set_value* trilogy_set_untag(trilogy_value* val);
trilogy_set_value* trilogy_set_assume(trilogy_value* val);

void trilogy_set_destroy(trilogy_set_value* arr);
