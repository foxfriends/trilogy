#pragma once
#include "types.h"

trilogy_value trilogy_struct(unsigned long i, trilogy_value* val);
trilogy_struct_value* untag_struct(trilogy_value* val);
trilogy_struct_value* assume_struct(trilogy_value* val);
void destroy_struct(trilogy_struct_value* val);
