; CMake grammar supplied by the tree-sitter-cmake feature.
(normal_command (identifier) @function)
[(quoted_argument) (bracket_argument)] @string
[(line_comment) (bracket_comment)] @comment
(variable) @variable
(escape_sequence) @string.escape
["$" "{" "}"] @punctuation.special
["(" ")"] @punctuation.bracket
[
  (function) (endfunction)
  (macro) (endmacro)
  (if) (elseif) (else) (endif)
  (foreach) (endforeach)
  (while) (endwhile)
] @keyword
(function_command (argument_list . (argument) @function))
(macro_command (argument_list . (argument) @function))
((unquoted_argument) @boolean
 (#match? @boolean "^(ON|OFF|TRUE|FALSE|YES|NO|on|off|true|false|yes|no)$"))
((unquoted_argument) @number
 (#match? @number "^-?[0-9]+(\\.[0-9]+)?$"))
