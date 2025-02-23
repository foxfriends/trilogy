#pragma once
#include "types.h"

void trilogy_number_init(trilogy_value* tv, trilogy_number_value i);
trilogy_number_value trilogy_number_untag(trilogy_value* val);
trilogy_number_value trilogy_number_assume(trilogy_value* val);

int trilogy_number_compare(trilogy_number_value lhs, trilogy_number_value rhs);
