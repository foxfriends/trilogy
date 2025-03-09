#pragma once
#include "types.h"
#include <stdint.h>

void trilogy_character_init(trilogy_value* t, uint32_t c);
uint32_t trilogy_character_untag(trilogy_value* val);
uint32_t trilogy_character_assume(trilogy_value* val);
int trilogy_character_compare(uint32_t lhs, uint32_t rhs);
