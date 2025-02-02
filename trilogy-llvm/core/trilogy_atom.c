#include "trilogy_atom.h"
#include "internal.h"
#include "runtime.h"
#include "trilogy_string.h"
#include "trilogy_value.h"
#include <assert.h>

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

void lookup_atom(trilogy_value* rv, trilogy_value* atom) {
    unsigned int atom_id = trilogy_atom_untag(atom);
    if (atom_id < atom_registry_sz) {
        trilogy_string_clone_into(rv, &atom_registry[atom_id]);
    } else {
        *rv = trilogy_unit;
    }
}
