"fn" @keyword
"let" @keyword
"return" @keyword
"if" @keyword
"match" @keyword
"effects" @keyword

(comment) @comment
(string_literal) @string
(number_literal) @number
(bool_literal) @boolean

(function_declaration name: (identifier) @function)
(function_declaration return_type: (type_identifier) @type)
(parameter name: (identifier) @variable.parameter)
(parameter type: (type_identifier) @type)
(effects_clause (effect_identifier) @attribute)
(let_statement (identifier) @variable)
(call_expression callee: (identifier) @function.call)
(call_expression callee: (member_expression property: (identifier) @function.call))
(member_expression property: (identifier) @property)

(identifier) @variable
