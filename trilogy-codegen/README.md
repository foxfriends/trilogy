# Trilogy Code Generation

Converts the [Trilogy IR](../trilogy-ir/) into bytecode to be interpreted by the
[Trilogy VM](../trilogy-vm/).

It turns out the IR is not all that optimal for conversion to bytecode at this time,
but we're rolling with it anyway. Another lower layer in between would have been
beneficial, that uses the raw continuation-based control flow instead of having to
convert high-level control flow to continuations by hand.
