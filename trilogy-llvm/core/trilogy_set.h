#pragma once
#include "types.h"

trilogy_value trilogy_set_empty();
trilogy_value trilogy_set_clone(trilogy_set_value* arr);
trilogy_set_value* untag_set(trilogy_value* val);
trilogy_set_value* assume_set(trilogy_value* val);
void trilogy_set_destroy(trilogy_set_value* arr);
