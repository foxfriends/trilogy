#pragma once
#include "types.h"
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

trilogy_bits_value*
trilogy_bits_init(trilogy_value* tv, trilogy_bits_value* bits);
trilogy_bits_value*
trilogy_bits_init_new(trilogy_value* tv, size_t len, uint8_t* b);
trilogy_bits_value*
trilogy_bits_clone_into(trilogy_value* tv, trilogy_bits_value* val);

trilogy_bits_value* trilogy_bits_untag(trilogy_value* val);
trilogy_bits_value* trilogy_bits_assume(trilogy_value* val);

size_t trilogy_bits_bytelen(trilogy_bits_value* val);
bool trilogy_bits_at(trilogy_bits_value* b, size_t index);
int trilogy_bits_compare(trilogy_bits_value* lhs, trilogy_bits_value* rhs);

void trilogy_bits_destroy(trilogy_bits_value* b);
