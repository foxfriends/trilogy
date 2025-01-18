#pragma once
#include "types.h"

trilogy_record_value* trilogy_record_empty(trilogy_value* tv, trilogy_record_value* rec);
trilogy_record_value* trilogy_record_init_empty(trilogy_value* tv);
trilogy_record_value* trilogy_record_clone_into(trilogy_value* tv, trilogy_record_value* rec);

trilogy_record_value* trilogy_record_untag(trilogy_value* val);
trilogy_record_value* trilogy_record_assume(trilogy_value* val);

void trilogy_record_destroy(trilogy_record_value* arr);
