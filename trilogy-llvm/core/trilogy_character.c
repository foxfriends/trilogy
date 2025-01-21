#include "trilogy_character.h"
#include "internal.h"
#include <stdlib.h>
#include <assert.h>

void trilogy_character_init(trilogy_value* t, unsigned int ch) {
    assert(t->tag == TAG_UNDEFINED);
    t->tag = TAG_CHAR;
    t->payload = (unsigned long)ch;
}

unsigned int trilogy_character_untag(trilogy_value* val) {
    if (val->tag != TAG_CHAR) rte("character", val->tag);
    return trilogy_character_assume(val);
}

unsigned int trilogy_character_assume(trilogy_value* val) { return (unsigned int)val->payload; }
