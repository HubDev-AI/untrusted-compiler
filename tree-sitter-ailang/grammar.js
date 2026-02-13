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
        field("name", $.identifier),
        "(",
        optional($.parameter_list),
        ")",
        optional($.effects_clause),
        optional(seq("->", field("return_type", $.type_identifier))),
        field("body", $.block)
      ),

    parameter_list: ($) => seq($.parameter, repeat(seq(",", $.parameter))),
    parameter: ($) => seq(field("name", $.identifier), ":", field("type", $.type_identifier)),
    type_identifier: ($) => $.identifier,
    effects_clause: ($) => seq("effects", "{", optional($.effect_list), "}"),
    effect_list: ($) => seq($.effect_identifier, repeat(seq(",", $.effect_identifier))),
    effect_identifier: () => /[A-Za-z_][A-Za-z0-9_.]*/,

    block: ($) => seq("{", repeat($.statement), optional($.expression), "}"),

    statement: ($) =>
      choice($.let_statement, $.return_statement, $.expression_statement),

    let_statement: ($) => seq("let", $.identifier, "=", $.expression),
    return_statement: ($) => seq("return", optional($.expression)),
    expression_statement: ($) => $.expression,

    expression: ($) =>
      choice(
        $.call_expression,
        $.member_expression,
        $.bool_literal,
        $.identifier,
        $.number_literal,
        $.string_literal,
        $.parenthesized_expression
      ),

    call_expression: ($) =>
      prec(
        1,
        seq(
          field("callee", choice($.identifier, $.member_expression)),
          "(",
          optional($.argument_list),
          ")"
        )
      ),
    member_expression: ($) =>
      prec.left(
        2,
        seq(
          field("object", choice($.identifier, $.member_expression)),
          ".",
          field("property", $.identifier)
        )
      ),

    argument_list: ($) => seq($.expression, repeat(seq(",", $.expression))),
    parenthesized_expression: ($) => seq("(", $.expression, ")"),

    bool_literal: () => choice("true", "false"),
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
