#include "hash.h"
#include "internal.h"
#include <stdint.h>
#include <stdlib.h>

// Implements FNV-1a hash function, according to Wikipedia

#define FNV_PRIME 1099511628211u
#define FNV_OFFSET_BASIS 14695981039346656037u

struct hasher {
    uint64_t hash;
};

hasher* hash_new() {
    hasher* hasher = malloc_safe(sizeof(hasher));
    hasher->hash = FNV_OFFSET_BASIS;
    return hasher;
}

void hash_update(hasher* h, uint8_t byte) {
    h->hash ^= byte;
    h->hash *= FNV_PRIME;
}

void hash_update_n(hasher* h, const uint8_t* bytes, size_t n) {
    for (size_t i = 0; i < n; ++i) {
        hash_update(h, bytes[i]);
    }
}

uint64_t hash_finish(hasher* h) {
    uint64_t hash = h->hash;
    free(h);
    return hash;
}
