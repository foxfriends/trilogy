#pragma once
#include "types.h"

void trace(trilogy_value* rt);
void panic(trilogy_value* rv, trilogy_value* message);
void print(trilogy_value* rv, trilogy_value* str);

void boolean_not(trilogy_value* rv, trilogy_value* v);
void boolean_and(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs);
void boolean_or(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs);

void bitwise_and(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs);
void bitwise_or(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs);
void bitwise_xor(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs);
void bitwise_invert(trilogy_value* rv, trilogy_value* val);

void shift_left(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs);
void shift_left_extend(
    trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs
);
void shift_left_contract(
    trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs
);

void referential_eq(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs);
void referential_neq(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs);
void structural_eq(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs);
void structural_neq(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs);

void add(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs);
void subtract(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs);
void multiply(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs);
void divide(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs);
void negate(trilogy_value* rv, trilogy_value* val);

void length(trilogy_value* rv, trilogy_value* arr);
void push(trilogy_value* rv, trilogy_value* arr, trilogy_value* val);
void append(trilogy_value* rv, trilogy_value* arr, trilogy_value* val);

void glue(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs);
void member_access(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs);
void cons(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs);

void compare(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs);
void lt(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs);
void gt(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs);
void lte(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs);
void gte(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs);

void lookup_atom(trilogy_value* rv, trilogy_value* atom);
void to_string(trilogy_value* rv, trilogy_value* val);
