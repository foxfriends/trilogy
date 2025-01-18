#pragma once
#include "types.h"

trilogy_value trilogy_character(unsigned int c);
unsigned int trilogy_character_untag(trilogy_value* val);
unsigned int trilogy_character_assume(trilogy_value* val);
