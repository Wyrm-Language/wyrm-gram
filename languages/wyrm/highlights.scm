; Wyrm highlights. Capture names follow Zed/Gram's theme vocabulary.

(comment) @comment

; ---- literals -------------------------------------------------------------
[(string) (raw_string)] @string
(escape_sequence) @string.escape
(character) @string.special
(symbol) @string.special.symbol
(integer) @number
(float) @number
[(true) (false)] @boolean
[(nil) (undefined)] @constant.builtin
(ellipsis) @constant.builtin

; ---- identifiers -------------------------------------------------------------
(identifier) @variable

(this) @variable.special
((identifier) @variable.special
  (#eq? @variable.special "$ast"))

(parameter name: (identifier) @variable.parameter)
(var_positional name: (identifier) @variable.parameter)
(var_keyword name: (identifier) @variable.parameter)
(keyword_argument name: (identifier) @variable.parameter)
(for_statement variable: (identifier) @variable)

(var_target name: (identifier) @variable)
(static_statement name: (identifier) @variable)
(with_binding name: (identifier) @variable)

; ---- namespaces and imports -------------------------------------------
(qualified_name (identifier) @namespace)
(qualified_name (identifier) @variable .)
(import_path (identifier) @namespace)
(import_item name: (identifier) @variable)
(import_item alias: (identifier) @variable)
(import_statement alias: (identifier) @variable)
(from_import_statement module: (import_path (identifier) @namespace))
(except_names (identifier) @variable)

; ---- types -----------------------------------------------------------------
(type_atom (identifier) @type)
(type_atom (qualified_name (identifier) @type .))
(type_atom (qualified_name (identifier) @namespace))
(class_target (identifier) @type)
(send_type "<-" @operator)

; Builtin type names read as types wherever they are ordinary identifiers.
((identifier) @type.builtin
  (#any-of? @type.builtin
    "bool" "int" "uint" "float" "sym" "bytes" "str" "tuple" "list" "dict" "pair" "array"
    "fiber" "closure" "coroutine" "message" "object" "error" "file"))
((identifier) @type
  (#any-of? @type "Unset" "NotFound" "OutOfMemory" "StopIteration" "RuntimeError" "OSError"))

; ---- definitions ------------------------------------------------------------
(function_definition name: (identifier) @function)
(coroutine_definition name: (identifier) @function)
(signal_definition name: (identifier) @function)
(class_definition name: (identifier) @type)
(class_definition superclasses: (arguments (identifier) @type))
(anonymous_class superclasses: (arguments (identifier) @type))
(slot_definition name: (identifier) @property)
(slot_option kind: _ @keyword)

; ---- calls -----------------------------------------------------------------
(call function: (identifier) @function)
(call function: (qualified_name (identifier) @function .))
(call function: (attribute attribute: (identifier) @function.method))
(attribute attribute: (identifier) @property)
(message selector: (identifier) @function.method)
(message "!" @operator)
(emit_statement name: (identifier) @function)
(super_call "super" @keyword)
(defined_expression "defined" @function.builtin)

(decorator "@" @attribute)
(decorator name: (identifier) @attribute)

((call function: (identifier) @function.builtin)
  (#any-of? @function.builtin "print" "len" "copy" "cons" "pair" "car" "cdr" "next" "send"))

; ---- keywords ----------------------------------------------------------
[
  "and" "or" "not" "in" "is"
  "try" "catch" "defer" "on"
  "if" "elif" "else" "while" "for"
  "return" "yield" "from"
  "do" "with" "task" "thread" "emit"
] @keyword

[
  "fn" "co" "class" "slot" "signal" "var" "static"
] @keyword

(break) @keyword
(continue) @keyword
(pass) @keyword

[
  "import" "as" "except"
] @keyword

[
  "getter" "setter"
] @keyword

; ---- operators and punctuation ---------------------------------------------
[
  "+" "-" "*" "/" "%" "**" "&" "|" "^" "~" "<<" ">>"
  "==" "!=" "<" ">" "<=" ">=" "<=>"
  "=" ":=" "?=" "->" "<-" "::"
] @operator

[ "(" ")" "[" "]" "{" "}" "$[" ] @punctuation.bracket
[ "," ";" ] @punctuation.delimiter
":" @punctuation.delimiter
"." @punctuation.delimiter
