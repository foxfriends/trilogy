#pragma once
#include "types.h"
#include <stdlib.h>

[[noreturn]] void internal_panic(char* msg);
[[noreturn]] void rte(char* expected, unsigned char tag);
[[noreturn]] void exit_(trilogy_value* code);
void* malloc_safe(size_t size);
void* calloc_safe(size_t num, size_t size);
void* realloc_safe(void* ptr, size_t size);

void trilogy_unhandled_effect(trilogy_value* effect);
void trilogy_execution_ended();
