#pragma once
#include "types.h"

trilogy_value trilogy_tuple(trilogy_value* fst, trilogy_value* snd);
trilogy_tuple_value* untag_tuple(trilogy_value* val);
trilogy_tuple_value* assume_tuple(trilogy_value* val);
void destroy_tuple(trilogy_tuple_value* val);
