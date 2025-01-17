#pragma once
#include <stdbool.h>
#include "trilogy_value.h"

void internal_panic(char* msg);
void rte(char* expected, unsigned char tag);
void exit_(struct trilogy_value* code);
bool is_structural_eq(
    struct trilogy_value* lhs,
    struct trilogy_value* rhs
);
