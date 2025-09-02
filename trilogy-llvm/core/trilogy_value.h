#pragma once
#include "types.h"
#include <stdbool.h>
#include <stdint.h>

extern const trilogy_value trilogy_undefined;
extern const trilogy_value trilogy_unit;
void trilogy_unit_untag(trilogy_value* val);

void trilogy_value_destroy(trilogy_value* val);
void trilogy_value_clone_into(trilogy_value* into, trilogy_value* from);
void trilogy_value_clone_undefined_into(
    trilogy_value* into, trilogy_value* from
);

bool trilogy_value_structural_eq(trilogy_value* lhs, trilogy_value* rhs);
bool trilogy_value_referential_eq(trilogy_value* lhs, trilogy_value* rhs);
int trilogy_value_compare(trilogy_value* lhs, trilogy_value* rhs);

void trilogy_value_to_string(trilogy_value* rv, trilogy_value* val);

uint64_t trilogy_value_hash(trilogy_value* value);
