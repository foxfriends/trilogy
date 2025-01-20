#pragma once
#include "types.h"

trilogy_set_value* trilogy_set_init(trilogy_value* tv, trilogy_set_value* set);
trilogy_set_value* trilogy_set_init_empty(trilogy_value* tv);
trilogy_set_value*
trilogy_set_clone_into(trilogy_value* tv, trilogy_set_value* arr);

trilogy_set_value* trilogy_set_untag(trilogy_value* val);
trilogy_set_value* trilogy_set_assume(trilogy_value* val);

void trilogy_set_destroy(trilogy_set_value* arr);
