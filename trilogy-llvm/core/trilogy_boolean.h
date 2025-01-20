#pragma once
#include "types.h"
#include <stdbool.h>

extern const trilogy_value trilogy_true;
extern const trilogy_value trilogy_false;
trilogy_value trilogy_boolean(bool b);
bool trilogy_boolean_untag(trilogy_value* val);
bool trilogy_boolean_assume(trilogy_value* val);
