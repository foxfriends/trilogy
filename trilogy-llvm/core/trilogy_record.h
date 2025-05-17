#pragma once
#include "types.h"
#include <stdbool.h>

trilogy_record_value*
trilogy_record_empty(trilogy_value* tv, trilogy_record_value* rec);
trilogy_record_value* trilogy_record_init_empty(trilogy_value* tv);
trilogy_record_value* trilogy_record_init_cap(trilogy_value* tv, size_t cap);
trilogy_record_value*
trilogy_record_clone_into(trilogy_value* tv, trilogy_record_value* rec);

void trilogy_record_insert(
    trilogy_record_value* record, trilogy_value* key, trilogy_value* value
);
void trilogy_record_delete(trilogy_record_value* record, trilogy_value* key);
void trilogy_record_get(
    trilogy_value* out, trilogy_record_value* record, trilogy_value* key
);
bool trilogy_record_contains_key(
    trilogy_record_value* record, trilogy_value* key
);

trilogy_record_value* trilogy_record_untag(trilogy_value* val);
trilogy_record_value* trilogy_record_assume(trilogy_value* val);

void trilogy_record_destroy(trilogy_record_value* rec);
