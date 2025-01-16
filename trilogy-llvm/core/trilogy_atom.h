#pragma once
#include "types.h"

trilogy_value trilogy_atom(unsigned long i);
unsigned long untag_atom(trilogy_value* val);
unsigned long assume_atom(trilogy_value* val);
