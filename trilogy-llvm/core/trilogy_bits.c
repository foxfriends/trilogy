#include "trilogy_bits.h"
#include "internal.h"
#include <assert.h>
#include <stdlib.h>
#include <string.h>

static size_t bit_len_to_byte_len(size_t n) { return n / 8 + (n & 7 ? 1 : 0); }

static trilogy_bits_value* trilogy_bits_new(size_t len, uint8_t* bytes) {
    trilogy_bits_value* bits = malloc_safe(sizeof(trilogy_bits_value));
    bits->len = len;
    bits->contents = bytes;
    return bits;
}

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
    size_t byte_len = bit_len_to_byte_len(len);
    bits->len = len;
    bits->contents = malloc_safe(sizeof(uint8_t) * byte_len);
    memcpy(bits->contents, b, byte_len);
    return trilogy_bits_init(tv, bits);
}

trilogy_bits_value*
trilogy_bits_clone_into(trilogy_value* tv, trilogy_bits_value* val) {
    trilogy_bits_value* bits = malloc_safe(sizeof(trilogy_bits_value));
    bits->len = val->len;
    bits->contents = malloc_safe(sizeof(uint8_t) * val->len);
    memcpy(bits->contents, val->contents, bit_len_to_byte_len(val->len));
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
    return bit_len_to_byte_len(val->len);
}

size_t trilogy_bits_len(trilogy_bits_value* val) { return val->len; }

bool trilogy_bits_eq(trilogy_bits_value* lhs, trilogy_bits_value* rhs) {
    if (lhs->len != rhs->len) return false;
    if (lhs->len == 0) return true;
    int cmp = memcmp(lhs->contents, rhs->contents, lhs->len / 8);
    if (cmp != 0) return false;
    size_t last_len = lhs->len % 8;
    if (last_len == 0) return true;
    size_t byte_len = bit_len_to_byte_len(lhs->len);
    uint8_t lhs_last = lhs->contents[byte_len - 1];
    uint8_t rhs_last = rhs->contents[byte_len - 1];
    uint8_t mask = ~0 >> (8 - last_len) << (8 - last_len);
    return (lhs_last & mask) == (rhs_last & mask);
}

int trilogy_bits_compare(trilogy_bits_value* lhs, trilogy_bits_value* rhs) {
    size_t len = lhs->len < rhs->len ? lhs->len : rhs->len;
    int cmp = memcmp(lhs->contents, rhs->contents, len / 8);
    if (cmp != 0) return cmp;
    size_t last_len = lhs->len % 8;
    if (last_len != 0) {
        size_t byte_len = bit_len_to_byte_len(len);
        uint8_t lhs_last = lhs->contents[byte_len - 1];
        uint8_t rhs_last = rhs->contents[byte_len - 1];
        uint8_t mask = ~0 >> (8 - last_len) << (8 - last_len);
        if ((lhs_last & mask) < (rhs_last & mask)) return -1;
        if ((lhs_last & mask) > (rhs_last & mask)) return 1;
    }
    if (lhs->len < rhs->len) return -1;
    if (lhs->len > rhs->len) return 1;
    return 0;
}

trilogy_bits_value*
trilogy_bits_and(trilogy_bits_value* lhs, trilogy_bits_value* rhs) {
    size_t lhs_len = trilogy_bits_bytelen(lhs);
    size_t rhs_len = trilogy_bits_bytelen(rhs);
    size_t bit_len = lhs->len > rhs->len ? lhs->len : rhs->len;
    size_t len = bit_len_to_byte_len(bit_len);
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
    size_t len = bit_len_to_byte_len(bit_len);
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
    size_t len = bit_len_to_byte_len(bit_len);
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

trilogy_bits_value*
trilogy_bits_shift_left_extend(trilogy_bits_value* lhs, size_t n) {
    assert(n != 0);
    size_t old_bit_len = lhs->len;
    size_t space = SIZE_MAX - old_bit_len;
    if (n > space) internal_panic("bits length limit\n");
    size_t new_bit_len = old_bit_len + n;
    size_t old_len = bit_len_to_byte_len(old_bit_len);
    size_t new_len = bit_len_to_byte_len(new_bit_len);

    uint8_t* out_bytes = malloc_safe(sizeof(uint8_t) * new_len);
    memcpy(out_bytes, lhs->contents, old_len);
    if (old_len != new_len) {
        memset(out_bytes + old_len, 0, new_len - old_len);
    }

    size_t old_tail_size = old_bit_len % 8;
    if (old_tail_size) {
        uint8_t overlap_bits = ~0 >> (8 - old_tail_size) << (8 - old_tail_size);
        out_bytes[old_len - 1] &= overlap_bits;
    }

    return trilogy_bits_new(new_bit_len, out_bytes);
}

static void shift_left_into(
    uint8_t* out, const uint8_t* in, const size_t byte_dist,
    const size_t bit_dist, const size_t n
) {
    for (size_t i = 0; i < n; ++i) {
        uint8_t left_part = in[i + byte_dist] << bit_dist;
        uint8_t right_part = 0;
        if (i + 1 < n) {
            right_part = in[i + byte_dist + 1] >> (8 - bit_dist);
        }
        out[i] = left_part | right_part;
    }
}

trilogy_bits_value*
trilogy_bits_shift_left_contract(trilogy_bits_value* lhs, size_t n) {
    assert(n != 0);
    assert(n <= lhs->len);
    size_t old_bit_len = lhs->len;
    size_t new_bit_len = old_bit_len - n;
    size_t new_len = bit_len_to_byte_len(new_bit_len);
    size_t byte_dist = n / 8;
    size_t bit_dist = n % 8;
    uint8_t* out_bytes = malloc_safe(sizeof(uint8_t) * new_len);
    shift_left_into(out_bytes, lhs->contents, byte_dist, bit_dist, new_len);
    return trilogy_bits_new(new_bit_len, out_bytes);
}

trilogy_bits_value* trilogy_bits_shift_left(trilogy_bits_value* lhs, size_t n) {
    assert(n != 0);
    assert(n <= lhs->len);
    size_t bit_len = lhs->len;
    size_t len = bit_len_to_byte_len(bit_len);
    size_t byte_dist = n / 8;
    size_t bit_dist = n % 8;
    uint8_t* out_bytes = malloc_safe(sizeof(uint8_t) * len);
    memset(out_bytes, 0, len);
    shift_left_into(
        out_bytes, lhs->contents, byte_dist, bit_dist, len - byte_dist
    );
    return trilogy_bits_new(bit_len, out_bytes);
}

static void shift_right_into(
    uint8_t* out, const uint8_t* in, const size_t byte_dist,
    const size_t bit_dist, const size_t n
) {
    for (size_t i = 0; i < n; ++i) {
        uint8_t left_part = 0;
        if (i > 0) left_part = in[i - 1] << (8 - bit_dist);
        uint8_t right_part = in[i] >> bit_dist;
        out[i + byte_dist] = left_part | right_part;
    }
}

trilogy_bits_value*
trilogy_bits_shift_right_extend(trilogy_bits_value* lhs, size_t n) {
    assert(n != 0);
    size_t old_bit_len = lhs->len;
    size_t space = SIZE_MAX - old_bit_len;
    if (n > space) internal_panic("bits length limit\n");
    size_t new_bit_len = old_bit_len + n;
    size_t old_len = bit_len_to_byte_len(old_bit_len);
    size_t new_len = bit_len_to_byte_len(new_bit_len);
    size_t byte_dist = n / 8;
    size_t bit_dist = n % 8;

    uint8_t* out_bytes = malloc_safe(sizeof(uint8_t) * new_len);
    memset(out_bytes, 0, new_len);
    shift_right_into(
        out_bytes, lhs->contents, byte_dist, bit_dist, new_len - byte_dist
    );
    return trilogy_bits_new(new_bit_len, out_bytes);
}

trilogy_bits_value*
trilogy_bits_shift_right_contract(trilogy_bits_value* lhs, size_t n) {
    assert(n != 0);
    assert(n <= lhs->len);
    size_t new_bit_len = lhs->len - n;
    size_t new_len = bit_len_to_byte_len(new_bit_len);
    uint8_t* out_bytes = malloc_safe(sizeof(uint8_t) * new_len);
    memcpy(out_bytes, lhs->contents, new_len);
    return trilogy_bits_new(new_bit_len, out_bytes);
}

trilogy_bits_value*
trilogy_bits_shift_right(trilogy_bits_value* lhs, size_t n) {
    assert(n != 0);
    size_t bit_len = lhs->len;
    size_t byte_len = bit_len_to_byte_len(bit_len);
    size_t byte_dist = n / 8;
    size_t bit_dist = n % 8;
    uint8_t* out_bytes = malloc_safe(sizeof(uint8_t) * byte_len);
    memset(out_bytes, 0, byte_len);
    shift_right_into(
        out_bytes, lhs->contents, byte_dist, bit_dist, byte_len - byte_dist
    );
    return trilogy_bits_new(bit_len, out_bytes);
}

trilogy_bits_value* trilogy_bits_assume(trilogy_value* val) {
    assert(val->tag == TAG_BITS);
    return (trilogy_bits_value*)val->payload;
}

void trilogy_bits_destroy(trilogy_bits_value* b) { free(b->contents); }
