#include <assert.h>
#include <stdlib.h>
#include "trilogy_array.h"
#include "trilogy_value.h"
#include "internal.h"

trilogy_array_value* trilogy_array_init(trilogy_value* tv, trilogy_array_value* arr) {
    tv->tag = TAG_ARRAY;
    tv->payload = (unsigned long)arr;
    return arr;
}

trilogy_array_value* trilogy_array_init_empty(trilogy_value* tv) {
    trilogy_array_value* arr = malloc(sizeof(trilogy_array_value));
    arr->rc = 1;
    arr->len = 0;
    arr->cap = 0;
    arr->contents = NULL;
    return trilogy_array_init(tv, arr);
}

trilogy_array_value* trilogy_array_init_cap(trilogy_value* tv, unsigned long cap) {
    trilogy_array_value* arr = malloc(sizeof(trilogy_array_value));
    arr->rc = 1;
    arr->len = 0;
    arr->cap = cap;
    arr->contents = malloc(sizeof(trilogy_value) * cap);
    return trilogy_array_init(tv, arr);
}

trilogy_array_value* trilogy_array_clone_into(trilogy_value* tv, trilogy_array_value* arr) {
    assert(arr->rc != 0);
    ++arr->rc;
    return trilogy_array_init(tv, arr);
}

trilogy_array_value* trilogy_array_untag(trilogy_value* val) {
    if (val->tag != TAG_ARRAY) rte("array", val->tag);
    return trilogy_array_assume(val);
}

trilogy_array_value* trilogy_array_assume(trilogy_value* val) {
    return (trilogy_array_value*)val->payload;
}

void trilogy_array_destroy(trilogy_array_value* arr) {
    if (--arr->rc == 0) {
        if (arr->contents == NULL) return;
        for (unsigned long i = 0; i < arr->len; ++i) {
            trilogy_value_destroy(&arr->contents[i]);
        }
        free(arr->contents);
    }
}
