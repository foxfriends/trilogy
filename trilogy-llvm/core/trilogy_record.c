#include "trilogy_record.h"
#include "internal.h"
#include "trilogy_tuple.h"
#include "trilogy_value.h"
#include "types.h"
#include <assert.h>
#include <stdint.h>
#include <stdlib.h>

// This is hash map with basic open-addressed linear probing.

// TODO: the hash SUCKS right now, but technically this should "function".
static size_t hash(trilogy_value* value) { return 0; }

trilogy_record_value*
trilogy_record_init(trilogy_value* tv, trilogy_record_value* rec) {
    assert(tv->tag == TAG_UNDEFINED);
    tv->tag = TAG_RECORD;
    tv->payload = (uint64_t)rec;
    return rec;
}

trilogy_record_value* trilogy_record_init_empty(trilogy_value* tv) {
    trilogy_record_value* record = malloc_safe(sizeof(trilogy_record_value));
    record->rc = 1;
    record->len = 0;
    record->cap = 0;
    record->contents = NULL;
    return trilogy_record_init(tv, record);
}

trilogy_record_value* trilogy_record_init_cap(trilogy_value* tv, size_t cap) {
    trilogy_record_value* record = malloc_safe(sizeof(trilogy_record_value));
    record->rc = 1;
    record->len = 0;
    record->cap = cap;
    record->contents =
        cap == 0 ? NULL : calloc_safe(cap, sizeof(trilogy_tuple_value));
    return trilogy_record_init(tv, record);
}

trilogy_record_value*
trilogy_record_clone_into(trilogy_value* tv, trilogy_record_value* record) {
    assert(record->rc != 0);
    ++record->rc;
    return trilogy_record_init(tv, record);
}

static size_t trilogy_record_find(
    trilogy_record_value* record, trilogy_value* key, size_t* insert_to
) {
    if (insert_to) *insert_to = record->cap;
    size_t h = hash(key) % record->cap;
    for (;; h = h == record->cap - 1 ? 0 : h + 1) {
        trilogy_tuple_value* entry = &record->contents[h];
        if (entry->fst.tag == TAG_UNDEFINED &&
            entry->snd.tag == TAG_UNDEFINED) {
            // Returning cap to indicate not found. Otherwise, return value is
            // in range. Insert here only if we haven't already found a better
            // spot.
            if (insert_to && *insert_to == record->cap) *insert_to = h;
            return record->cap;
        }
        if (entry->fst.tag == TAG_UNDEFINED) {
            // Key unset, but value not undefined: entry was deleted. Skip it as
            // if it were filled, since it might have been filled at time of
            // insert. We can insert here if the value is not found later, and
            // we haven't already found a better spot.
            if (insert_to && *insert_to == record->cap) *insert_to = h;
            continue;
        }
        if (trilogy_value_structural_eq(key, &entry->fst)) {
            if (insert_to) *insert_to = h;
            return h;
        }
    }
}

static size_t trilogy_record_maintainance(trilogy_record_value* record) {
    // Maximum load factor = 75%
    if (record->len >= record->cap - record->cap / 4) {
        size_t old_cap = record->cap;
        trilogy_tuple_value* old_contents = record->contents;
        size_t new_cap = old_cap <= SIZE_MAX / 2 ? old_cap * 2 : SIZE_MAX;
        if (new_cap == 0) new_cap = 8;
        record->cap = new_cap;
        record->len = 0;
        record->contents = calloc_safe(new_cap, sizeof(trilogy_tuple_value));
        for (size_t i = 0; i < old_cap; ++i) {
            trilogy_record_insert(
                record, &old_contents[i].fst, &old_contents[i].snd
            );
        }
        free(old_contents);
    }
}

void trilogy_record_insert(
    trilogy_record_value* record, trilogy_value* key, trilogy_value* value
) {
    trilogy_record_maintainance(record);
    size_t empty = record->cap;
    size_t found = trilogy_record_find(record, key, &empty);
    if (found == record->cap) {
        // If it's not found, insert the new key and value at the empty
        // position.
        record->contents[empty].fst = *key;
        record->contents[empty].snd = *value;
        record->len++;
    } else {
        // If it is found, delete the new key, destroy the old value, and then
        // insert the new value.
        trilogy_value_destroy(key);
        trilogy_value_destroy(&record->contents[found].snd);
        record->contents[found].snd = *value;
    }
}

void trilogy_record_delete(trilogy_record_value* record, trilogy_value* key) {
    size_t found = trilogy_record_find(record, key, NULL);
    if (found != record->cap) {
        // Only if it's found does it need to be destroyed. Remove the key (to
        // mark as empty), destroy the value, and then store something not
        // undefined in the value (we'll use unit) to indicate that it's a
        // deleted item and not just an unused cell.
        trilogy_value_destroy(&record->contents[found].fst);
        trilogy_value_destroy(&record->contents[found].snd);
        record->contents[found].fst = trilogy_undefined;
        record->contents[found].snd = trilogy_unit;
    }
}

bool trilogy_record_contains_key(
    trilogy_record_value* record, trilogy_value* key
) {
    return trilogy_record_find(record, key, NULL) != record->cap;
}

void trilogy_record_get(
    trilogy_value* out, trilogy_record_value* record, trilogy_value* key
) {
    size_t found = trilogy_record_find(record, key, NULL);
    if (found == record->cap) internal_panic("key not found in record\n");
    trilogy_value_clone_into(out, &record->contents[found].snd);
}

trilogy_record_value* trilogy_record_untag(trilogy_value* val) {
    if (val->tag != TAG_RECORD) rte("record", val->tag);
    return trilogy_record_assume(val);
}

trilogy_record_value* trilogy_record_assume(trilogy_value* val) {
    assert(val->tag == TAG_RECORD);
    return (trilogy_record_value*)val->payload;
}

void trilogy_record_destroy(trilogy_record_value* record) {
    if (--record->rc == 0) {
        if (record->contents == NULL) return;
        for (size_t i = 0; i < record->len; ++i) {
            trilogy_tuple_destroy(&record->contents[i]);
        }
        free(record->contents);
        free(record);
    }
}
