module.exports = grammar({
  name: "ailang",

  extras: ($) => [/\s/, $.comment],
  word: ($) => $.identifier,

  rules: {
    source_file: ($) => repeat($.item),

    item: ($) => choice($.function_declaration),

    function_declaration: ($) =>
      seq(
        "fn",
        $.identifier,
        "(",
        optional($.parameter_list),
        ")",
        optional(seq("->", $.type_identifier)),
        $.block
      ),

    parameter_list: ($) => seq($.parameter, repeat(seq(",", $.parameter))),
    parameter: ($) => seq($.identifier, ":", $.type_identifier),
    type_identifier: ($) => $.identifier,

    block: ($) => seq("{", repeat($.statement), optional($.expression), "}"),

    statement: ($) =>
      choice($.let_statement, $.return_statement, $.expression_statement),

    let_statement: ($) => seq("let", $.identifier, "=", $.expression),
    return_statement: ($) => seq("return", optional($.expression)),
    expression_statement: ($) => $.expression,

    expression: ($) =>
      choice(
        $.call_expression,
        $.identifier,
        $.number_literal,
        $.string_literal,
        $.parenthesized_expression
      ),

    call_expression: ($) =>
      prec(1, seq($.identifier, "(", optional($.argument_list), ")")),

    argument_list: ($) => seq($.expression, repeat(seq(",", $.expression))),
    parenthesized_expression: ($) => seq("(", $.expression, ")"),

    number_literal: () => /[0-9]+/,
    string_literal: () => /"[^"\n]*"/,
    identifier: () => /[A-Za-z_][A-Za-z0-9_]*/,

    comment: () =>
      token(
        choice(
          seq("//", /.*/),
          seq("/*", /[^*]*\*+([^/*][^*]*\*+)*/, "/")
        )
      ),
  },
});
