#pragma once
#include "internal.h"
#include "types.h"

void trace(trilogy_value* rt);
void panic(trilogy_value* rv, trilogy_value* message);
void print(trilogy_value* rv, trilogy_value* str);

void structural_eq(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs);
void referential_eq(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs);
