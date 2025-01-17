#pragma once
#include "types.h"

trilogy_value trilogy_struct_new(unsigned long i, trilogy_value* val);
trilogy_value trilogy_struct_clone(trilogy_struct_value* val);
trilogy_struct_value* untag_struct(trilogy_value* val);
trilogy_struct_value* assume_struct(trilogy_value* val);
void trilogy_struct_destroy(trilogy_struct_value* val);
