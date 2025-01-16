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

trilogy_bits_value* untag_bits(trilogy_value* val) {
    if (val->tag != TAG_BITS) rte("bits", val->tag);
    return assume_bits(val);
}

trilogy_bits_value* assume_bits(trilogy_value* val) {
    return (trilogy_bits_value*)val->payload;
}

void destroy_bits(trilogy_bits_value* b) {
    free(b->contents);
}
