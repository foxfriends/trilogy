#include <assert.h>
#include <stdlib.h>
#include "trilogy_set.h"
#include "trilogy_value.h"
#include "internal.h"

trilogy_value trilogy_set_empty() {
    trilogy_set_value* set = malloc(sizeof(trilogy_set_value));
    set->rc = 1;
    set->len = 0;
    set->cap = 0;
    set->contents = NULL;
    trilogy_value t = { .tag = TAG_SET, .payload = (unsigned long)set };
    return t;
}

trilogy_value trilogy_set_clone(trilogy_set_value* set) {
    assert(set->rc != 0);
    ++set->rc;
    trilogy_value t = { .tag = TAG_SET, .payload = (unsigned long)set };
    return t;
}

trilogy_set_value* trilogy_set_untag(trilogy_value* val) {
    if (val->tag != TAG_SET) rte("set", val->tag);
    return trilogy_set_assume(val);
}

trilogy_set_value* trilogy_set_assume(trilogy_value* val) {
    return (trilogy_set_value*)val->payload;
}

void trilogy_set_destroy(trilogy_set_value* set) {
    if (--set->rc == 0) {
        if (set->contents == NULL) return;
        for (unsigned long i = 0; i < set->len; ++i) {
            trilogy_value_destroy(&set->contents[i]);
        }
        free(set->contents);
    }
}
