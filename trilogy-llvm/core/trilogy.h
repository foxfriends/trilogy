#pragma once
#include "types.h"
#include "internal.h"

void panic(
    struct trilogy_value* rv,
    struct trilogy_value* message
);

void print(
    struct trilogy_value* rv,
    struct trilogy_value* str
);
