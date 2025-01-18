#pragma once
#include "types.h"

trilogy_bits_value* trilogy_bits_init(trilogy_value* tv, trilogy_bits_value* bits);
trilogy_bits_value* trilogy_bits_init_new(trilogy_value* tv, unsigned long len, unsigned char* b);
trilogy_bits_value* trilogy_bits_clone_into(trilogy_value* tv, trilogy_bits_value* val);

trilogy_bits_value* trilogy_bits_untag(trilogy_value* val);
trilogy_bits_value* trilogy_bits_assume(trilogy_value* val);

void trilogy_bits_destroy(trilogy_bits_value* b);
