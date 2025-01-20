#pragma once

#include "types.h"

[[noreturn]] void internal_panic(char* msg);
[[noreturn]] void rte(char* expected, unsigned char tag);
[[noreturn]] void exit_(trilogy_value* code);
