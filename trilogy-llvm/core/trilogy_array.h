#pragma once
#include "types.h"

trilogy_array_value* trilogy_array_init(trilogy_value* tv, trilogy_array_value* arr);
trilogy_array_value* trilogy_array_init_empty(trilogy_value* tv);
trilogy_array_value* trilogy_array_init_cap(trilogy_value* tv, unsigned long cap);
trilogy_array_value* trilogy_array_clone_into(trilogy_value* tv, trilogy_array_value* arr);

trilogy_array_value* trilogy_array_untag(trilogy_value* val);
trilogy_array_value* trilogy_array_assume(trilogy_value* val);

void trilogy_array_destroy(trilogy_array_value* arr);
