#pragma once
#include "types.h"

void trace(trilogy_value* rt);
void panic(trilogy_value* rv, trilogy_value* message);
void print(trilogy_value* rv, trilogy_value* str);
void readline(trilogy_value* rv);
void readchar(trilogy_value* rv);

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

void shift_right(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs);
void shift_right_extend(
    trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs
);
void shift_right_contract(
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
void int_divide(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs);
void rem(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs);
void power(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs);
void negate(trilogy_value* rv, trilogy_value* val);

void length(trilogy_value* rv, trilogy_value* arr);
void push(trilogy_value* rv, trilogy_value* arr, trilogy_value* val);
void pop(trilogy_value* rv, trilogy_value* arr);
void append(trilogy_value* rv, trilogy_value* arr, trilogy_value* val);
void contains_key(trilogy_value* rv, trilogy_value* arr, trilogy_value* key);

void glue(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs);
void unglue_start(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs);
void unglue_end(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs);

void cons(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs);

void member_access(trilogy_value* rv, trilogy_value* lhs, trilogy_value* index);
void member_assign(
    trilogy_value* rv, trilogy_value* lhs, trilogy_value* index,
    trilogy_value* value
);

void compare(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs);
void lt(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs);
void gt(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs);
void lte(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs);
void gte(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs);

void lookup_atom(trilogy_value* rv, trilogy_value* atom);
void to_string(trilogy_value* rv, trilogy_value* val);

void set_to_array(trilogy_value* rv, trilogy_value* val);
void record_to_array(trilogy_value* rv, trilogy_value* val);
void string_to_array(trilogy_value* rv, trilogy_value* val);

void re(trilogy_value* rv, trilogy_value* num);
void im(trilogy_value* rv, trilogy_value* num);
void numer(trilogy_value* rv, trilogy_value* num);
void denom(trilogy_value* rv, trilogy_value* num);
