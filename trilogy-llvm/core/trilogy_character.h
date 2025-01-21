#pragma once
#include "types.h"

void trilogy_character_init(trilogy_value* t, unsigned int c);
unsigned int trilogy_character_untag(trilogy_value* val);
unsigned int trilogy_character_assume(trilogy_value* val);
