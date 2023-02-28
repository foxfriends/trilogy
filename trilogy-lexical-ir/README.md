# Trilogy Lexical Intermedate Representation

The Trilogy Lexical IR is a simplified AST maintaining the lexical structure of
the original Trilogy program. The goal is to provide a single target language on
which to perform further analysis, rather than having to analyze all three of
Trilogy's sub-languages separately.

During the construction of this IR is when name resolution takes place.
