#include "trilogy_reference.h"
#include "trilogy_value.h"
#include <assert.h>
#include <stdlib.h>

trilogy_reference*
trilogy_reference_init(trilogy_value* tv, trilogy_reference* ref) {
    assert(tv->tag == TAG_UNDEFINED);
    tv->tag = TAG_REFERENCE;
    tv->payload = (unsigned long)ref;
    return ref;
}

trilogy_reference* trilogy_reference_to(trilogy_value* tv, trilogy_value* val) {
    trilogy_reference* ref = malloc(sizeof(trilogy_reference));
    ref->rc = 1;
    ref->location = val;
    ref->closed = trilogy_undefined;
    return trilogy_reference_init(tv, ref);
}

trilogy_reference*
trilogy_reference_clone_into(trilogy_value* tv, trilogy_reference* ref) {
    assert(ref->rc != 0);
    ref->rc++;
    return trilogy_reference_init(tv, ref);
}

void trilogy_reference_close(trilogy_reference* ref) {
    assert(ref->location != &ref->closed);
    ref->closed = *ref->location;
    ref->location = &ref->closed;
}

trilogy_reference* trilogy_reference_assume(trilogy_value* val) {
    return (trilogy_reference*)val->payload;
}

void trilogy_reference_destroy(trilogy_reference* ref) {
    if (--ref->rc == 0) {
        assert(ref->location == &ref->closed);
        trilogy_value_destroy(&ref->closed);
        free(ref);
    }
}
