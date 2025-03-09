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
    tv->payload = (uint64_t)arr;
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

trilogy_array_value* trilogy_array_init_cap(trilogy_value* tv, uint64_t cap) {
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

uint64_t trilogy_array_len(trilogy_array_value* arr) { return arr->len; }

uint64_t trilogy_array_cap(trilogy_array_value* arr) { return arr->cap; }

uint64_t __trilogy_array_resize(trilogy_array_value* arr, uint64_t cap) {
    trilogy_value* new_contents = calloc_safe(cap, sizeof(trilogy_value));
    memcpy(new_contents, arr->contents, sizeof(trilogy_value) * arr->len);
    free(arr->contents);
    arr->cap = cap;
    arr->contents = new_contents;
    return cap;
}

uint64_t trilogy_array_resize(trilogy_array_value* arr, uint64_t cap) {
    if (cap < arr->len) cap = arr->len;
    return __trilogy_array_resize(arr, cap);
}

uint64_t trilogy_array_reserve(trilogy_array_value* arr, uint64_t to_reserve) {
    uint64_t space = arr->cap - arr->len;
    if (space >= to_reserve) return arr->cap;
    uint64_t max_claimable = ULONG_MAX - arr->cap;
    if (to_reserve > max_claimable) internal_panic("array capacity limit\n");
    if (to_reserve < arr->cap) to_reserve = arr->cap;
    if (to_reserve > max_claimable) to_reserve = max_claimable;
    return __trilogy_array_resize(arr, arr->cap + to_reserve);
}

void trilogy_array_push(trilogy_array_value* arr, trilogy_value* tv) {
    trilogy_array_reserve(arr, 1);
    arr->contents[arr->len] = *tv;
    *tv = trilogy_undefined;
    ++arr->len;
}

void trilogy_array_append(trilogy_array_value* arr, trilogy_value* tv) {
    trilogy_array_value* tail = trilogy_array_untag(tv);
    uint64_t tail_len = trilogy_array_len(tail);
    trilogy_array_reserve(arr, tail_len);
    if (tail->rc == 1) {
        memcpy(
            arr->contents + arr->len, tail->contents,
            sizeof(trilogy_value) * tail_len
        );
        free(tail->contents);
        free(tail);
        *tv = trilogy_undefined;
    } else {
        for (uint64_t i = 0; i < tail_len; ++i) {
            trilogy_value_clone_into(
                &arr->contents[arr->len + i], &tail->contents[i]
            );
        }
        trilogy_value_destroy(tv);
    }
    arr->len += tail_len;
}

void trilogy_array_at(
    trilogy_value* tv, trilogy_array_value* arr, uint64_t index
) {
    assert(index < arr->len);
    trilogy_value_clone_into(tv, &arr->contents[index]);
}

int trilogy_array_compare(trilogy_array_value* lhs, trilogy_array_value* rhs) {
    uint64_t len = lhs->len < rhs->len ? lhs->len : rhs->len;
    for (uint64_t i = 0; i < len; ++i) {
        int cmp = trilogy_value_compare(&lhs->contents[i], &rhs->contents[i]);
        if (cmp != 0) return cmp;
    }
    if (lhs->len < rhs->len) return -1;
    if (lhs->len > rhs->len) return 1;
    return 0;
}

trilogy_array_value* trilogy_array_untag(trilogy_value* val) {
    if (val->tag != TAG_ARRAY) rte("array", val->tag);
    return trilogy_array_assume(val);
}

trilogy_array_value* trilogy_array_assume(trilogy_value* val) {
    assert(val->tag == TAG_ARRAY);
    return (trilogy_array_value*)val->payload;
}

void trilogy_array_destroy(trilogy_array_value* arr) {
    if (--arr->rc == 0) {
        if (arr->contents != NULL) {
            for (uint64_t i = 0; i < arr->len; ++i) {
                trilogy_value_destroy(&arr->contents[i]);
            }
            free(arr->contents);
        }
        free(arr);
    }
}
