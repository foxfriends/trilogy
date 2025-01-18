#pragma once
#include "types.h"

trilogy_value trilogy_struct_new(unsigned long i, trilogy_value* val);
trilogy_value trilogy_struct_clone(trilogy_struct_value* val);
trilogy_struct_value* trilogy_struct_untag(trilogy_value* val);
trilogy_struct_value* trilogy_struct_assume(trilogy_value* val);
void trilogy_struct_destroy(trilogy_struct_value* val);
