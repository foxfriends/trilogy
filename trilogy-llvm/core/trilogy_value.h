#pragma once
#include "types.h"

void destroy_trilogy_value(trilogy_value* val);

extern const trilogy_value trilogy_undefined;
extern const trilogy_value trilogy_unit;
void untag_unit(trilogy_value* val);
