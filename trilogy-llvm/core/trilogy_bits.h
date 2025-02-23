#pragma once
#include "types.h"
#include <stdbool.h>

trilogy_bits_value*
trilogy_bits_init(trilogy_value* tv, trilogy_bits_value* bits);
trilogy_bits_value*
trilogy_bits_init_new(trilogy_value* tv, unsigned long len, unsigned char* b);
trilogy_bits_value*
trilogy_bits_clone_into(trilogy_value* tv, trilogy_bits_value* val);

trilogy_bits_value* trilogy_bits_untag(trilogy_value* val);
trilogy_bits_value* trilogy_bits_assume(trilogy_value* val);

bool trilogy_bits_at(trilogy_bits_value* b, unsigned long index);
int trilogy_bits_compare(trilogy_bits_value* lhs, trilogy_bits_value* rhs);

void trilogy_bits_destroy(trilogy_bits_value* b);
