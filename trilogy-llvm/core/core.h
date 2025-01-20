#pragma once
#include "types.h"
#include "internal.h"

void trace(struct trilogy_value* rt);
void panic(struct trilogy_value* rv, struct trilogy_value* message);
void print(struct trilogy_value* rv, struct trilogy_value* str);

void structural_eq(struct trilogy_value* rv, struct trilogy_value* lhs, struct trilogy_value* rhs);
void referential_eq(struct trilogy_value* rv, struct trilogy_value* lhs, struct trilogy_value* rhs);
