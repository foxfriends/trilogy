#include "trilogy_set.h"
#include "internal.h"
#include "trilogy_value.h"
#include <assert.h>
#include <stdint.h>
#include <stdlib.h>

trilogy_set_value* trilogy_set_init(trilogy_value* tv, trilogy_set_value* set) {
    assert(tv->tag == TAG_UNDEFINED);
    tv->tag = TAG_SET;
    tv->payload = (uint64_t)set;
    return set;
}

trilogy_set_value* trilogy_set_init_empty(trilogy_value* tv) {
    trilogy_set_value* set = malloc_safe(sizeof(trilogy_set_value));
    set->rc = 1;
    set->len = 0;
    set->cap = 0;
    set->contents = NULL;
    return trilogy_set_init(tv, set);
}

trilogy_set_value*
trilogy_set_clone_into(trilogy_value* tv, trilogy_set_value* set) {
    assert(set->rc != 0);
    ++set->rc;
    return trilogy_set_init(tv, set);
}

trilogy_set_value* trilogy_set_untag(trilogy_value* val) {
    if (val->tag != TAG_SET) rte("set", val->tag);
    return trilogy_set_assume(val);
}

trilogy_set_value* trilogy_set_assume(trilogy_value* val) {
    assert(val->tag == TAG_SET);
    return (trilogy_set_value*)val->payload;
}

void trilogy_set_destroy(trilogy_set_value* set) {
    if (--set->rc == 0) {
        if (set->contents == NULL) return;
        for (uint64_t i = 0; i < set->len; ++i) {
            trilogy_value_destroy(&set->contents[i]);
        }
        free(set->contents);
        free(set);
    }
}
