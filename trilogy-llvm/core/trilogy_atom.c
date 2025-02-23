#include "trilogy_atom.h"
#include "trilogy_value.h"
#include "internal.h"
#include "runtime.h"
#include <assert.h>
#include <stddef.h>

void trilogy_atom_init(trilogy_value* t, unsigned long i) {
    assert(t->tag == TAG_UNDEFINED);
    t->tag = TAG_ATOM;
    t->payload = i;
}

unsigned long trilogy_atom_untag(trilogy_value* val) {
    if (val->tag != TAG_ATOM) rte("atom", val->tag);
    return trilogy_atom_assume(val);
}

unsigned long trilogy_atom_assume(trilogy_value* val) {
    assert(val->tag == TAG_ATOM);
    return (unsigned long)val->payload;
}

const trilogy_string_value* trilogy_atom_repr(unsigned long atom_id) {
    if (atom_id < atom_registry_sz) {
        return &atom_registry[atom_id];
    } else {
        return NULL;
    }
}

void trilogy_atom_make_cmp(trilogy_value* rv, int cmp) {
    switch (cmp) {
    case -1: return trilogy_atom_init(rv, ATOM_LT);
    case 0: return trilogy_atom_init(rv, ATOM_EQ);
    case 1: return trilogy_atom_init(rv, ATOM_GT);
    default: *rv = trilogy_unit;
    }
}
