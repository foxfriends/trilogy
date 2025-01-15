#pragma once
#include "types.h"
#include "internal.h"

void trilogy_panic(
    struct trilogy_value* rv,
    struct trilogy_value* message
);

void trilogy_exit(
    struct trilogy_value* rv,
    struct trilogy_value* code
);

void trilogy_printf(
    struct trilogy_value* rv,
    struct trilogy_value* str
);

void trilogy_structural_eq(
    struct trilogy_value* rv,
    struct trilogy_value* lhs,
    struct trilogy_value* rhs
);

void trilogy_lookup_atom(
    struct trilogy_value* rv,
    struct trilogy_value* atom
);
