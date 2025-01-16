#pragma once
#include "types.h"

trilogy_value trilogy_string_new(unsigned long len, char* s);
trilogy_value trilogy_string_from_c(char* s);
char* trilogy_string_to_c(trilogy_string_value* val);
trilogy_string_value* untag_string(trilogy_value* val);
trilogy_string_value* assume_string(trilogy_value* val);
void destroy_string(trilogy_string_value* val);
