#pragma once
#include "types.h"

trilogy_value trilogy_record_empty();
trilogy_value trilogy_record_clone(trilogy_record_value* arr);
trilogy_record_value* untag_record(trilogy_value* val);
trilogy_record_value* assume_record(trilogy_value* val);
void destroy_record(trilogy_record_value* arr);
