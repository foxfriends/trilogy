#pragma once
#include "types.h"

trilogy_value trilogy_atom(unsigned long i);
unsigned long trilogy_atom_untag(trilogy_value* val);
unsigned long trilogy_atom_assume(trilogy_value* val);
void lookup_atom(trilogy_value* rv, trilogy_value* atom);
