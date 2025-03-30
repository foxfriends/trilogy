#include "trilogy_bits.h"
#include "internal.h"
#include <assert.h>
#include <stdlib.h>
#include <string.h>

trilogy_bits_value*
trilogy_bits_init(trilogy_value* tv, trilogy_bits_value* bits) {
    assert(tv->tag == TAG_UNDEFINED);
    tv->tag = TAG_BITS;
    tv->payload = (uint64_t)bits;
    return bits;
}

trilogy_bits_value*
trilogy_bits_init_new(trilogy_value* tv, size_t len, uint8_t* b) {
    trilogy_bits_value* bits = malloc_safe(sizeof(trilogy_bits_value));
    bits->len = len;
    bits->contents = malloc_safe(sizeof(uint8_t) * len);
    memcpy(bits->contents, b, len);
    return trilogy_bits_init(tv, bits);
}

trilogy_bits_value*
trilogy_bits_clone_into(trilogy_value* tv, trilogy_bits_value* val) {
    trilogy_bits_value* bits = malloc_safe(sizeof(trilogy_bits_value));
    bits->len = val->len;
    bits->contents = malloc_safe(sizeof(uint8_t) * val->len);
    memcpy(bits->contents, val->contents, val->len);
    return trilogy_bits_init(tv, bits);
}

trilogy_bits_value* trilogy_bits_untag(trilogy_value* val) {
    if (val->tag != TAG_BITS) rte("bits", val->tag);
    return trilogy_bits_assume(val);
}

bool trilogy_bits_at(trilogy_bits_value* b, size_t index) {
    assert(index <= b->len);
    uint8_t byte = b->contents[index >> 3];
    return (bool)(1 & (byte >> (7 - (index & 7))));
}

size_t trilogy_bits_bytelen(trilogy_bits_value* val) {
    size_t len = val->len / 8;
    if (val->len & 7) len++;
    return len;
}

int trilogy_bits_compare(trilogy_bits_value* lhs, trilogy_bits_value* rhs) {
    size_t lhs_len = trilogy_bits_bytelen(lhs);
    size_t rhs_len = trilogy_bits_bytelen(rhs);
    size_t len = lhs_len < rhs_len ? lhs_len : rhs_len;
    int cmp = memcmp(lhs->contents, rhs->contents, len);
    if (cmp != 0) return cmp;
    if (lhs->len < rhs->len) return -1;
    if (lhs->len > rhs->len) return 1;
    return 0;
}

trilogy_bits_value*
trilogy_bits_and(trilogy_bits_value* lhs, trilogy_bits_value* rhs) {
    size_t lhs_len = trilogy_bits_bytelen(lhs);
    size_t rhs_len = trilogy_bits_bytelen(rhs);
    size_t bit_len = lhs->len > rhs->len ? lhs->len : rhs->len;
    size_t len = bit_len / 8 + (bit_len & 7 ? 1 : 0);
    uint8_t* out_bytes = malloc_safe(sizeof(uint8_t) * len);
    for (size_t i = 0; i < len; ++i) {
        uint8_t lb = i < lhs_len ? lhs->contents[i] : 0;
        uint8_t rb = i < rhs_len ? rhs->contents[i] : 0;
        out_bytes[i] = lb & rb;
    }
    trilogy_bits_value* bits = malloc_safe(sizeof(trilogy_bits_value));
    bits->len = bit_len;
    bits->contents = out_bytes;
    return bits;
}

trilogy_bits_value*
trilogy_bits_or(trilogy_bits_value* lhs, trilogy_bits_value* rhs) {
    size_t lhs_len = trilogy_bits_bytelen(lhs);
    size_t rhs_len = trilogy_bits_bytelen(rhs);
    size_t bit_len = lhs->len > rhs->len ? lhs->len : rhs->len;
    size_t len = bit_len / 8 + (bit_len & 7 ? 1 : 0);
    uint8_t* out_bytes = malloc_safe(sizeof(uint8_t) * len);
    for (size_t i = 0; i < len; ++i) {
        uint8_t lb = i < lhs_len ? lhs->contents[i] : 0;
        uint8_t rb = i < rhs_len ? rhs->contents[i] : 0;
        out_bytes[i] = lb | rb;
    }
    trilogy_bits_value* bits = malloc_safe(sizeof(trilogy_bits_value));
    bits->len = bit_len;
    bits->contents = out_bytes;
    return bits;
}

trilogy_bits_value*
trilogy_bits_xor(trilogy_bits_value* lhs, trilogy_bits_value* rhs) {
    size_t lhs_len = trilogy_bits_bytelen(lhs);
    size_t rhs_len = trilogy_bits_bytelen(rhs);
    size_t bit_len = lhs->len > rhs->len ? lhs->len : rhs->len;
    size_t len = bit_len / 8 + (bit_len & 7 ? 1 : 0);
    uint8_t* out_bytes = malloc_safe(sizeof(uint8_t) * len);
    for (size_t i = 0; i < len; ++i) {
        uint8_t lb = i < lhs_len ? lhs->contents[i] : 0;
        uint8_t rb = i < rhs_len ? rhs->contents[i] : 0;
        out_bytes[i] = lb ^ rb;
    }
    trilogy_bits_value* bits = malloc_safe(sizeof(trilogy_bits_value));
    bits->len = bit_len;
    bits->contents = out_bytes;
    return bits;
}

trilogy_bits_value* trilogy_bits_assume(trilogy_value* val) {
    assert(val->tag == TAG_BITS);
    return (trilogy_bits_value*)val->payload;
}

void trilogy_bits_destroy(trilogy_bits_value* b) { free(b->contents); }
