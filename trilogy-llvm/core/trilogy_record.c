#include <assert.h>
#include <stdlib.h>
#include "trilogy_record.h"
#include "trilogy_tuple.h"
#include "internal.h"

trilogy_value trilogy_record_empty() {
    trilogy_record_value* record = malloc(sizeof(trilogy_record_value));
    record->rc = 1;
    record->len = 0;
    record->cap = 0;
    record->contents = NULL;
    trilogy_value t = { .tag = TAG_RECORD, .payload = (unsigned long)record };
    return t;
}

trilogy_value trilogy_record_clone(trilogy_record_value* record) {
    assert(record->rc != 0);
    ++record->rc;
    trilogy_value t = { .tag = TAG_RECORD, .payload = (unsigned long)record };
    return t;
}

trilogy_record_value* untag_record(trilogy_value* val) {
    if (val->tag != TAG_RECORD) rte("record", val->tag);
    return assume_record(val);
}

trilogy_record_value* assume_record(trilogy_value* val) {
    return (trilogy_record_value*)val->payload;
}

void trilogy_record_destroy(trilogy_record_value* record) {
    if (--record->rc == 0) {
        if (record->contents == NULL) return;
        for (unsigned long i = 0; i < record->len; ++i) {
            trilogy_tuple_destroy(&record->contents[i]);
        }
        free(record->contents);
    }
}
