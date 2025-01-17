#pragma once
#include <stdbool.h>
#include "types.h"

extern const trilogy_value trilogy_true;
extern const trilogy_value trilogy_false;
trilogy_value trilogy_boolean(bool b);
bool untag_boolean(trilogy_value* val);
bool assume_boolean(trilogy_value* val);
