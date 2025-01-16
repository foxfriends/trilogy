#pragma once
#include <stdbool.h>
#include "types.h"

extern const trilogy_value trilogy_true;
extern const trilogy_value trilogy_false;
trilogy_value trilogy_bool(bool b);
bool untag_bool(trilogy_value* val);
bool assume_bool(trilogy_value* val);
