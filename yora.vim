syn keyword functions print exit
syn region strings start="\"" end="\""
syn region comments start="/\*" end="\*/"
syn match comments "//.*$"

let b:current_syntax = "yora"

hi def link functions	GruvboxBlue
hi def link strings		GruvboxGreen
hi def link comments	GruvboxGray