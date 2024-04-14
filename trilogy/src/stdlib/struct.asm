	CLOSE &"core::return"
	DESTRUCT
	COPY
	CONST 'module
	VALEQ
	PJUMPF &"panic::invalid_call"
	POP
	COPY
	CONST 1
	VALEQ
	PJUMPF &"panic::incorrect_arity"
	POP
"struct export construct":
	LOADL 0
	CONST 'construct
	VALEQ
	JUMPF &"struct export destruct"
	CONST &"struct::construct"
	RETURN

"struct export destruct":
	LOADL 0
	CONST 'destruct
	VALEQ
	PJUMPF &"struct unresolved import"
	CONST &"struct::destruct"
	RETURN

"struct unresolved import":
	LOADL 0
	CONST ['construct,'destruct,]
	CONS
	CONST 'UnresolvedImport
	CONSTRUCT
	PANIC

"struct::construct":
	DESTRUCT
	COPY
	CONST 'procedure
	VALEQ
	PJUMPF &"panic::invalid_call"
	POP
	COPY
	CONST 2
	VALEQ
	PJUMPF &"panic::incorrect_arity"
	POP
	LOADL 0
	TYPEOF
	COPY
	CONST 'atom
	VALEQ
	PJUMPF &"panic::runtime_type_error"
	POP
	SWAP
	CONSTRUCT
	RETURN

"struct::destruct":
	DESTRUCT
	COPY
	CONST 'function
	VALEQ
	PJUMPF &"panic::invalid_call"
	POP
	COPY
	CONST 1
	VALEQ
	PJUMPF &"panic::incorrect_arity"
	POP
	DESTRUCT
	SWAP
	CONS
	RETURN
