#pragma once
#include <stdint.h>
#include <stdlib.h>

typedef struct hasher hasher;

hasher* hash_new();
void hash_update(hasher* h, uint8_t byte);
void hash_update_n(hasher* h, const uint8_t* bytes, size_t n);
uint64_t hash_finish(hasher* h);
