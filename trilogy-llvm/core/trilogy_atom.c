#include "trilogy_atom.h"
#include "internal.h"
#include "runtime.h"
#include "trilogy_value.h"
#include "types.h"
#include <assert.h>
#include <stddef.h>
#include <stdint.h>

void trilogy_atom_init(trilogy_value* t, uint64_t i) {
    assert(t->tag == TAG_UNDEFINED);
    t->tag = TAG_ATOM;
    t->payload = i;
}

uint64_t trilogy_atom_untag(trilogy_value* val) {
    if (val->tag != TAG_ATOM) rte("atom", val->tag);
    return trilogy_atom_assume(val);
}

uint64_t trilogy_atom_assume(trilogy_value* val) {
    assert(val->tag == TAG_ATOM);
    return (uint64_t)val->payload;
}

const trilogy_string_value* trilogy_atom_repr(uint64_t atom_id) {
    if (atom_id < atom_registry_sz) {
        return &atom_registry[atom_id];
    }
    return NULL;
}

void trilogy_atom_make_cmp(trilogy_value* rv, int cmp) {
    switch (cmp) {
    case -1:
        return trilogy_atom_init(rv, ATOM_LT);
    case 0:
        return trilogy_atom_init(rv, ATOM_EQ);
    case 1:
        return trilogy_atom_init(rv, ATOM_GT);
    default:
        *rv = trilogy_unit;
    }
}
