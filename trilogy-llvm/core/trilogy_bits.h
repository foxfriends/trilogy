#pragma once
#include "types.h"

trilogy_value trilogy_bits_new(unsigned long len, unsigned char* b);
trilogy_bits_value* untag_bits(trilogy_value* val);
trilogy_bits_value* assume_bits(trilogy_value* val);
void destroy_bits(trilogy_bits_value* b);
