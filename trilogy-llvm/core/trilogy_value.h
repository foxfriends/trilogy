#pragma once
#include "types.h"

extern const trilogy_value trilogy_undefined;
extern const trilogy_value trilogy_unit;
void untag_unit(trilogy_value* val);

void trilogy_value_destroy(trilogy_value* val);
trilogy_value trilogy_value_clone(trilogy_value* val);
void trilogy_value_clone_into(trilogy_value* into, trilogy_value* from);

void structural_eq(
    struct trilogy_value* rv,
    struct trilogy_value* lhs,
    struct trilogy_value* rhs
);
