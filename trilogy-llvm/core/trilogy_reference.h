#pragma once
#include "types.h"

trilogy_reference*
trilogy_reference_init(trilogy_value* tv, trilogy_reference* ref);
trilogy_reference* trilogy_reference_to(trilogy_value* tv, trilogy_value* val);
trilogy_reference* trilogy_reference_init_empty(trilogy_value* tv);
trilogy_reference*
trilogy_reference_clone_into(trilogy_value* tv, trilogy_reference* ref);

void trilogy_reference_close(trilogy_reference* ref);

trilogy_reference* trilogy_reference_assume(trilogy_value* val);

void trilogy_reference_destroy(trilogy_reference* ref);
