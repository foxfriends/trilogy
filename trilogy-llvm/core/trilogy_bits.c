#include "trilogy_bits.h"
#include "internal.h"
#include <assert.h>
#include <stdlib.h>
#include <string.h>

trilogy_bits_value*
trilogy_bits_init(trilogy_value* tv, trilogy_bits_value* bits) {
    assert(tv->tag == TAG_UNDEFINED);
    tv->tag = TAG_BITS;
    tv->payload = (unsigned long)bits;
    return bits;
}

trilogy_bits_value*
trilogy_bits_init_new(trilogy_value* tv, unsigned long len, unsigned char* b) {
    trilogy_bits_value* bits = malloc_safe(sizeof(trilogy_bits_value));
    bits->len = len;
    bits->contents = malloc_safe(sizeof(unsigned char) * len);
    memcpy(bits->contents, b, len);
    return trilogy_bits_init(tv, bits);
}

trilogy_bits_value*
trilogy_bits_clone_into(trilogy_value* tv, trilogy_bits_value* val) {
    trilogy_bits_value* bits = malloc_safe(sizeof(trilogy_bits_value));
    bits->len = val->len;
    bits->contents = malloc_safe(sizeof(unsigned char) * val->len);
    memcpy(bits->contents, val->contents, val->len);
    return trilogy_bits_init(tv, bits);
}

trilogy_bits_value* trilogy_bits_untag(trilogy_value* val) {
    if (val->tag != TAG_BITS) rte("bits", val->tag);
    return trilogy_bits_assume(val);
}

trilogy_bits_value* trilogy_bits_assume(trilogy_value* val) {
    return (trilogy_bits_value*)val->payload;
}

void trilogy_bits_destroy(trilogy_bits_value* b) { free(b->contents); }
