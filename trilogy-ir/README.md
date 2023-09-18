# Trilogy IR

A more compiler-friendly version of the AST. Created by removing syntax sugar,
insignificant language features, and resolving everything into unambiguous
identifiers. The resulting structure still largely reads like the original
program, but the lines between the three sub-languages are starting to blur.

This format remains largely per-file, with references to identifiers in other
files to be resolved later on.
