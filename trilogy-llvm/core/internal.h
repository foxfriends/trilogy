#pragma once

struct trilogy_value;

[[noreturn]] void internal_panic(char* msg);
[[noreturn]] void rte(char* expected, unsigned char tag);
[[noreturn]] void exit_(struct trilogy_value* code);
