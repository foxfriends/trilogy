#include "trilogy_character.h"
#include "internal.h"
#include <assert.h>

void trilogy_character_init(trilogy_value* t, uint32_t ch) {
    assert(t->tag == TAG_UNDEFINED);
    t->tag = TAG_CHAR;
    t->payload = (uint64_t)ch;
}

uint32_t trilogy_character_untag(trilogy_value* val) {
    if (val->tag != TAG_CHAR) rte("character", val->tag);
    return trilogy_character_assume(val);
}

uint32_t trilogy_character_assume(trilogy_value* val) {
    assert(val->tag == TAG_CHAR);
    return (uint32_t)val->payload;
}

int trilogy_character_compare(uint32_t lhs, uint32_t rhs) {
    if (lhs < rhs) return -1;
    if (lhs > rhs) return 1;
    return 0;
}
