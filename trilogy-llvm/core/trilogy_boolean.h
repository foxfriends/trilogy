#pragma once
#include "types.h"
#include <stdbool.h>

extern const trilogy_value trilogy_true;
extern const trilogy_value trilogy_false;
void trilogy_boolean_init(trilogy_value* t, bool b);
bool trilogy_boolean_untag(trilogy_value* val);
bool trilogy_boolean_assume(trilogy_value* val);
