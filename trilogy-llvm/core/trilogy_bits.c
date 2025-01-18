#include <stdlib.h>
#include <string.h>
#include "trilogy_bits.h"
#include "internal.h"

trilogy_value trilogy_bits_new(unsigned long len, unsigned char* b) {
    trilogy_bits_value* bits = malloc(sizeof(trilogy_bits_value));
    bits->len = len;
    bits->contents = malloc(sizeof(unsigned char) * len);
    memcpy(bits->contents, b, len);
    trilogy_value t = { .tag = TAG_BITS, .payload = (unsigned long)bits };
    return t;
}

trilogy_value trilogy_bits_clone(trilogy_bits_value* val) {
    trilogy_bits_value* bits = malloc(sizeof(trilogy_bits_value));
    bits->len = val->len;
    bits->contents = malloc(sizeof(unsigned char) * val->len);
    memcpy(bits->contents, val->contents, val->len);
    trilogy_value t = { .tag = TAG_BITS, .payload = (unsigned long)bits };
    return t;
}

trilogy_bits_value* trilogy_bits_untag(trilogy_value* val) {
    if (val->tag != TAG_BITS) rte("bits", val->tag);
    return trilogy_bits_assume(val);
}

trilogy_bits_value* trilogy_bits_assume(trilogy_value* val) {
    return (trilogy_bits_value*)val->payload;
}

void trilogy_bits_destroy(trilogy_bits_value* b) {
    free(b->contents);
}
