#pragma once
#include <stdbool.h>
#include "trilogy_value.h"

void trace(struct trilogy_value* rt);

[[noreturn]] void internal_panic(char* msg);
[[noreturn]] void rte(char* expected, unsigned char tag);
[[noreturn]] void exit_(struct trilogy_value* code);

bool is_structural_eq(
    struct trilogy_value* lhs,
    struct trilogy_value* rhs
);
