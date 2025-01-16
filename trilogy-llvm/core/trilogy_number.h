#pragma once
#include "types.h"

trilogy_value trilogy_integer(long i);
long untag_integer(trilogy_value* val);
long assume_integer(trilogy_value* val);
