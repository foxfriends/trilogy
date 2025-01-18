#pragma once
#include "types.h"

trilogy_value trilogy_integer(long i);
long trilogy_integer_untag(trilogy_value* val);
long trilogy_integer_assume(trilogy_value* val);
