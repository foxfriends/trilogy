#include "trilogy_atom.h"
#include "internal.h"
#include "runtime.h"
#include "trilogy_value.h"

trilogy_value trilogy_atom(unsigned long i) {
    trilogy_value t = {.tag = TAG_ATOM, .payload = i};
    return t;
}

unsigned long trilogy_atom_untag(trilogy_value* val) {
    if (val->tag != TAG_ATOM) rte("atom", val->tag);
    return trilogy_atom_assume(val);
}

unsigned long trilogy_atom_assume(trilogy_value* val) {
    return (unsigned long)val->payload;
}

void lookup_atom(trilogy_value* rv, trilogy_value* atom) {
    unsigned int atom_id = trilogy_atom_untag(atom);
    if (atom_id < atom_registry_sz) {
        rv->tag = TAG_STRING;
        rv->payload = (unsigned long)&atom_registry[atom_id];
    } else {
        *rv = trilogy_unit;
    }
}
