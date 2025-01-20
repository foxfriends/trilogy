#pragma once
#include <stdbool.h>
#include "types.h"

extern const trilogy_value trilogy_undefined;
extern const trilogy_value trilogy_unit;
void trilogy_unit_untag(trilogy_value* val);

void trilogy_value_destroy(trilogy_value* val);
void trilogy_value_clone_into(trilogy_value* into, trilogy_value* from);

bool trilogy_value_structural_eq(struct trilogy_value* lhs, struct trilogy_value* rhs);
bool trilogy_value_referential_eq(struct trilogy_value* lhs, struct trilogy_value* rhs);
