#pragma once
#include "types.h"

trilogy_value trilogy_character(unsigned int c);
unsigned int untag_character(trilogy_value* val);
unsigned int assume_character(trilogy_value* val);
