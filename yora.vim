syn keyword functions print exit
syn region comments start="/\*" end="\*/"
syn match comments "//.*$"

let b:current_syntax = "yora"

hi def link functions	Statement
hi def link comments	Comment
