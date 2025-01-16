#include "trilogy_atom.h"
#include "trilogy_value.h"
#include "internal.h"
#include "runtime.h"

trilogy_value trilogy_atom(unsigned long i) {
    trilogy_value t = { .tag = TAG_ATOM, .payload = i };
    return t;
}

unsigned long untag_atom(trilogy_value* val) {
    if (val->tag != TAG_ATOM) rte("atom", val->tag);
    return assume_atom(val);
}

unsigned long assume_atom(trilogy_value* val) {
    return (unsigned long)val->payload;
}

void trilogy_lookup_atom(
    struct trilogy_value* rv,
    struct trilogy_value* atom
) {
    unsigned int atom_id = untag_atom(atom);
    if (atom_id < atom_registry_sz) {
        rv->tag = TAG_STRING;
        rv->payload = (unsigned long)&atom_registry[atom_id];
    } else {
        *rv = trilogy_unit;
    }
}
