#pragma once
#include "types.h"

trilogy_value trilogy_array_empty();
trilogy_value trilogy_array_clone(trilogy_array_value* arr);
trilogy_array_value* untag_array(trilogy_value* val);
trilogy_array_value* assume_array(trilogy_value* val);
void trilogy_array_destroy(trilogy_array_value* arr);
