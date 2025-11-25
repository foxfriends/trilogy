#pragma once
#include "types.h"
#include <stdint.h>

#define ATOM_LEFT 15
#define ATOM_RIGHT 16

#define ATOM_LT 17
#define ATOM_EQ 18
#define ATOM_GT 19

#define ATOM_EOF 20
#define ATOM_ASSERTION_FAILED 21

void trilogy_atom_init(trilogy_value* t, uint64_t i);
uint64_t trilogy_atom_untag(trilogy_value* val);
uint64_t trilogy_atom_assume(trilogy_value* val);

const trilogy_string_value* trilogy_atom_repr(uint64_t i);
void trilogy_atom_make_cmp(trilogy_value* rv, int cmp);
