#include "trilogy_array.h"
#include "internal.h"
#include "trilogy_value.h"
#include <assert.h>
#include <limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

trilogy_array_value*
trilogy_array_init(trilogy_value* tv, trilogy_array_value* arr) {
    assert(tv->tag == TAG_UNDEFINED);
    tv->tag = TAG_ARRAY;
    tv->payload = (unsigned long)arr;
    return arr;
}

trilogy_array_value* trilogy_array_init_empty(trilogy_value* tv) {
    trilogy_array_value* arr = malloc_safe(sizeof(trilogy_array_value));
    arr->rc = 1;
    arr->len = 0;
    arr->cap = 0;
    arr->contents = NULL;
    return trilogy_array_init(tv, arr);
}

trilogy_array_value*
trilogy_array_init_cap(trilogy_value* tv, unsigned long cap) {
    trilogy_array_value* arr = malloc_safe(sizeof(trilogy_array_value));
    arr->rc = 1;
    arr->len = 0;
    arr->cap = cap;
    arr->contents = cap == 0 ? NULL : calloc_safe(cap, sizeof(trilogy_value));
    return trilogy_array_init(tv, arr);
}

trilogy_array_value*
trilogy_array_clone_into(trilogy_value* tv, trilogy_array_value* arr) {
    assert(arr->rc != 0);
    ++arr->rc;
    return trilogy_array_init(tv, arr);
}

unsigned long trilogy_array_len(trilogy_array_value* arr) { return arr->len; }

unsigned long trilogy_array_cap(trilogy_array_value* arr) { return arr->cap; }

unsigned long
__trilogy_array_resize(trilogy_array_value* arr, unsigned long cap) {
    trilogy_value* new_contents = calloc_safe(cap, sizeof(trilogy_value));
    memcpy(new_contents, arr->contents, sizeof(trilogy_value) * arr->len);
    free(arr->contents);
    arr->cap = cap;
    arr->contents = new_contents;
    return cap;
}

unsigned long
trilogy_array_resize(trilogy_array_value* arr, unsigned long cap) {
    if (cap < arr->len) cap = arr->len;
    return __trilogy_array_resize(arr, cap);
}

unsigned long
trilogy_array_reserve(trilogy_array_value* arr, unsigned long to_reserve) {
    unsigned long space = arr->cap - arr->len;
    if (space >= to_reserve) return arr->cap;
    unsigned long max_claimable = ULONG_MAX - arr->cap;
    if (to_reserve > max_claimable) internal_panic("array limit");
    if (to_reserve < arr->cap) to_reserve = arr->cap;
    if (to_reserve > max_claimable) to_reserve = max_claimable;
    return __trilogy_array_resize(arr, arr->cap + to_reserve);
}

void trilogy_array_push(trilogy_array_value* arr, trilogy_value* tv) {
    trilogy_array_reserve(arr, 1);
    trilogy_value_clone_into(&arr->contents[arr->len], tv);
    ++arr->len;
}

void trilogy_array_append(trilogy_array_value* arr, trilogy_value* tv) {
    trilogy_array_value* tail = trilogy_array_untag(tv);
    unsigned long tail_len = trilogy_array_len(tail);
    trilogy_array_reserve(arr, tail_len);
    for (unsigned long i = 0; i < tail_len; ++i) {
        trilogy_value_clone_into(
            &arr->contents[arr->len + i], &tail->contents[i]
        );
    }
    arr->len += tail_len;
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
        free(arr);
    }
}
