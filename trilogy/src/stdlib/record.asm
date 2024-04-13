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
"record export delete":
	LOADL 0
	CONST 'delete
	VALEQ
	JUMPF &"record export delete"
	CONST &"record::delete"
	RETURN

"record unresolved import":
	LOADL 0
	CONST ['delete,]
	CONS
	CONST 'UnresolvedImport
	CONSTRUCT
	PANIC

"record::delete":
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
	CONST 'record
	VALEQ
	PJUMPF &"panic::runtime_type_error"
	POP
	DELETE
	RETURN
