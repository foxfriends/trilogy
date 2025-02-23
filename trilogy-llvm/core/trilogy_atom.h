#pragma once
#include "types.h"

#define ATOM_LEFT 14
#define ATOM_RIGHT 15

#define ATOM_LT 16
#define ATOM_EQ 17
#define ATOM_GT 18

void trilogy_atom_init(trilogy_value* t, unsigned long i);
unsigned long trilogy_atom_untag(trilogy_value* val);
unsigned long trilogy_atom_assume(trilogy_value* val);

const trilogy_string_value* trilogy_atom_repr(unsigned long i);
void trilogy_atom_make_cmp(trilogy_value* rv, int cmp);
