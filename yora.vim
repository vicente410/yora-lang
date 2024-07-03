syn keyword functions print exit
syn keyword keywords var if else loop while continue break 
syn match operator "=\|+\|-\|*\|/\|%\|!\|and\|or\|<\|>"
syn match number '\d\+'
syn keyword boolean true false
syn region strings start="\"" end="\""
"syn region comments start="/\*" end="\*/"
syn match comments "#.*$"

let b:current_syntax = "yora"

hi def link functions	@function
hi def link keywords	@keyword
hi def link strings		@string
hi def link comments	@comment
hi def link operator	@operator
hi def link number		@number
hi def link boolean		@boolean
