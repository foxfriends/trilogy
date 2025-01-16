#include <assert.h>
#include <stdlib.h>
#include "trilogy_array.h"
#include "trilogy_value.h"
#include "internal.h"

trilogy_value trilogy_array_empty() {
    trilogy_array_value* arr = malloc(sizeof(trilogy_array_value));
    arr->rc = 1;
    arr->len = 0;
    arr->cap = 0;
    arr->contents = NULL;
    trilogy_value t = { .tag = TAG_ARRAY, .payload = (unsigned long)arr };
    return t;
}

trilogy_value trilogy_array_clone(trilogy_array_value* arr) {
    assert(arr->rc != 0);
    ++arr->rc;
    trilogy_value t = { .tag = TAG_ARRAY, .payload = (unsigned long)arr };
    return t;
}

trilogy_array_value* untag_array(trilogy_value* val) {
    if (val->tag != TAG_ARRAY) rte("array", val->tag);
    return assume_array(val);
}

trilogy_array_value* assume_array(trilogy_value* val) {
    return (trilogy_array_value*)val->payload;
}

void destroy_array(trilogy_array_value* arr) {
    if (--arr->rc == 0) {
        if (arr->contents == NULL) return;
        for (unsigned long i = 0; i < arr->len; ++i) {
            destroy_trilogy_value(&arr->contents[i]);
        }
        free(arr->contents);
    }
}
