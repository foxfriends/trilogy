#pragma once
#include "types.h"

trilogy_value trilogy_number(long i);
long trilogy_number_untag(trilogy_value* val);
long trilogy_number_assume(trilogy_value* val);
