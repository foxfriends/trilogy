#pragma once
#include "types.h"

trilogy_value trilogy_bits_new(unsigned long len, unsigned char* b);
trilogy_value trilogy_bits_clone(trilogy_bits_value* val);
trilogy_bits_value* untag_bits(trilogy_value* val);
trilogy_bits_value* assume_bits(trilogy_value* val);
void trilogy_bits_destroy(trilogy_bits_value* b);
