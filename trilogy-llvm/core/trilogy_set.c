#include "trilogy_set.h"
#include "internal.h"
#include "trilogy_value.h"
#include "types.h"
#include <assert.h>
#include <stdint.h>
#include <stdlib.h>

// This is hash map with basic open-addressed linear probing, all values are
// `unit` as it is a set.

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

trilogy_set_value* trilogy_set_init_cap(trilogy_value* tv, size_t cap) {
    trilogy_set_value* set = malloc_safe(sizeof(trilogy_set_value));
    set->rc = 1;
    set->len = 0;
    set->cap = cap;
    set->contents =
        cap == 0 ? NULL : calloc_safe(cap, sizeof(trilogy_tuple_value));
    return trilogy_set_init(tv, set);
}

trilogy_set_value*
trilogy_set_clone_into(trilogy_value* tv, trilogy_set_value* set) {
    assert(set->rc != 0);
    ++set->rc;
    return trilogy_set_init(tv, set);
}

static size_t trilogy_set_find(
    trilogy_set_value* set, trilogy_value* key, size_t* insert_to
) {
    if (insert_to) *insert_to = set->cap;
    size_t h = ((size_t)trilogy_value_hash(key)) % set->cap;
    for (;; h = h == set->cap - 1 ? 0 : h + 1) {
        trilogy_tuple_value* entry = &set->contents[h];
        if (entry->fst.tag == TAG_UNDEFINED &&
            entry->snd.tag == TAG_UNDEFINED) {
            // Returning cap to indicate not found. Otherwise, return value is
            // in range. Insert here only if we haven't already found a better
            // spot.
            if (insert_to && *insert_to == set->cap) *insert_to = h;
            return set->cap;
        }
        if (entry->fst.tag == TAG_UNDEFINED) {
            // Key unset, but value not undefined: entry was deleted. Skip it as
            // if it were filled, since it might have been filled at time of
            // insert. We can insert here if the value is not found later, and
            // we haven't already found a better spot.
            if (insert_to && *insert_to == set->cap) *insert_to = h;
            continue;
        }
        if (trilogy_value_structural_eq(key, &entry->fst)) {
            if (insert_to) *insert_to = h;
            return h;
        }
    }
}

static void trilogy_set_maintainance(trilogy_set_value* set) {
    // Maximum load factor = 75%
    if (set->len >= set->cap - set->cap / 4) {
        size_t old_cap = set->cap;
        trilogy_tuple_value* old_contents = set->contents;
        size_t new_cap = old_cap <= SIZE_MAX / 2 ? old_cap * 2 : SIZE_MAX;
        if (new_cap == 0) new_cap = 8;
        set->cap = new_cap;
        set->len = 0;
        set->contents = calloc_safe(new_cap, sizeof(trilogy_tuple_value));
        for (size_t i = 0; i < old_cap; ++i) {
            if (old_contents[i].fst.tag != TAG_UNDEFINED) {
                trilogy_set_insert(set, &old_contents[i].fst);
            }
        }
        free(old_contents);
    }
}

void trilogy_set_insert(trilogy_set_value* set, trilogy_value* value) {
    trilogy_set_maintainance(set);
    size_t empty = set->cap;
    size_t found = trilogy_set_find(set, value, &empty);
    if (found == set->cap) {
        // If it's not found, insert the new value and mark it with a `unit`.
        set->contents[empty].fst = *value;
        set->contents[empty].snd = trilogy_unit;
        set->len++;
        *value = trilogy_undefined;
    } else {
        // If it is found, delete the new value as if consumed, but we don't
        // have to change anything about the set's contents.
        trilogy_value_destroy(value);
    }
}

void trilogy_set_append(trilogy_set_value* set, trilogy_value* tv) {
    trilogy_set_value* tail = trilogy_set_untag(tv);
    uint64_t tail_len = tail->len;
    if (tail->rc == 1) {
        for (uint64_t i = 0; i < tail->cap; ++i) {
            trilogy_tuple_value* entry = &tail->contents[i];
            if (entry->fst.tag != TAG_UNDEFINED) {
                trilogy_set_insert(set, &entry->fst);
            }
        }
    } else {
        for (uint64_t i = 0; i < tail_len; ++i) {
            trilogy_value clone = trilogy_undefined;
            trilogy_value_clone_into(&clone, &tail->contents[i].fst);
            trilogy_set_insert(set, &clone);
        }
    }
    trilogy_value_destroy(tv);
}

void trilogy_set_delete(trilogy_set_value* set, trilogy_value* value) {
    size_t found = trilogy_set_find(set, value, NULL);
    if (found != set->cap) {
        // Only if it's found does it need to be destroyed. Remove the value (to
        // mark as empty), and destroy the it. The marker `unit` does not need
        // to be adjusted.
        trilogy_value_destroy(&set->contents[found].fst);
        set->len--;
    }
}

bool trilogy_set_contains(trilogy_set_value* set, trilogy_value* value) {
    return trilogy_set_find(set, value, NULL) != set->cap;
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
        for (size_t i = 0; i < set->len; ++i) {
            trilogy_value_destroy(&set->contents[i].fst);
        }
        free(set->contents);
        free(set);
    }
}

bool trilogy_set_structural_eq(trilogy_set_value* lhs, trilogy_set_value* rhs) {
    if (lhs->len != rhs->len) return false;
    for (uint64_t i = 0; i < lhs->cap; ++i) {
        trilogy_tuple_value* entry = &lhs->contents[i];
        if (entry->fst.tag == TAG_UNDEFINED) continue;
        size_t rhs_index = trilogy_set_find(rhs, &entry->fst, NULL);
        if (rhs_index == rhs->cap) return false;
    }
    return true;
}
