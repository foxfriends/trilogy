#pragma once
#include "types.h"

trilogy_value trilogy_record_empty();
trilogy_value trilogy_record_clone(trilogy_record_value* arr);
trilogy_record_value* trilogy_record_untag(trilogy_value* val);
trilogy_record_value* trilogy_record_assume(trilogy_value* val);
void trilogy_record_destroy(trilogy_record_value* arr);
