#pragma once
#include "types.h"

trilogy_struct_value*
trilogy_struct_init(trilogy_value* tv, trilogy_struct_value* st);
trilogy_struct_value*
trilogy_struct_init_new(trilogy_value* tv, unsigned long i, trilogy_value* val);
trilogy_struct_value*
trilogy_struct_clone_into(trilogy_value* tv, trilogy_struct_value* val);

trilogy_struct_value* trilogy_struct_untag(trilogy_value* val);
trilogy_struct_value* trilogy_struct_assume(trilogy_value* val);

void trilogy_struct_destroy(trilogy_struct_value* val);
