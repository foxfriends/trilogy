#pragma once
#include "types.h"

#define ATOM_LEFT 14
#define ATOM_RIGHT 15

void trilogy_atom_init(trilogy_value* t, unsigned long i);
unsigned long trilogy_atom_untag(trilogy_value* val);
unsigned long trilogy_atom_assume(trilogy_value* val);
void lookup_atom(trilogy_value* rv, trilogy_value* atom);
