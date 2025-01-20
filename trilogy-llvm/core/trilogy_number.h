#pragma once
#include "types.h"

void trilogy_number_init(trilogy_value* tv, long i);
long trilogy_number_untag(trilogy_value* val);
long trilogy_number_assume(trilogy_value* val);
