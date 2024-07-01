syn keyword functions print exit
syn keyword keywords var if else then loop while continue break and or
syn region strings start="\"" end="\""
syn region comments start="/\*" end="\*/"
syn match comments "//.*$"

let b:current_syntax = "yora"

hi def link functions	function
hi def link keywords	keyword
hi def link strings		string
hi def link comments	comment
