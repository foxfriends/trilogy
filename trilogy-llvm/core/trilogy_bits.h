#pragma once
#include "types.h"

trilogy_value trilogy_bits_new(unsigned long len, unsigned char* b);
trilogy_value trilogy_bits_clone(trilogy_bits_value* val);
trilogy_bits_value* trilogy_bits_untag(trilogy_value* val);
trilogy_bits_value* trilogy_bits_assume(trilogy_value* val);
void trilogy_bits_destroy(trilogy_bits_value* b);
