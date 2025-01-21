#include "trilogy_record.h"
#include "internal.h"
#include "trilogy_tuple.h"
#include <assert.h>
#include <stdlib.h>

trilogy_record_value*
trilogy_record_init(trilogy_value* tv, trilogy_record_value* rec) {
    assert(tv->tag == TAG_UNDEFINED);
    tv->tag = TAG_RECORD;
    tv->payload = (unsigned long)rec;
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

trilogy_record_value*
trilogy_record_clone_into(trilogy_value* tv, trilogy_record_value* record) {
    assert(record->rc != 0);
    ++record->rc;
    return trilogy_record_init(tv, record);
}

trilogy_record_value* trilogy_record_untag(trilogy_value* val) {
    if (val->tag != TAG_RECORD) rte("record", val->tag);
    return trilogy_record_assume(val);
}

trilogy_record_value* trilogy_record_assume(trilogy_value* val) {
    return (trilogy_record_value*)val->payload;
}

void trilogy_record_destroy(trilogy_record_value* record) {
    if (--record->rc == 0) {
        if (record->contents == NULL) return;
        for (unsigned long i = 0; i < record->len; ++i) {
            trilogy_tuple_destroy(&record->contents[i]);
        }
        free(record->contents);
        free(record);
    }
}
