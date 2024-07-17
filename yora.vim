syn keyword keyword var if else loop while continue break pr fn return
syn match operator "=\|+\|-\|*\|/\|%\|!\|<\|>"
syn keyword operator and or
syn match number '\d\+'
syn keyword boolean true false
syn region string start="\"" end="\""
syn region character start="\'" end="\'"
syn keyword type Bool Char Int
syn match function "\<\h\w*\ze\_s\{-}(\%(\*\h\w*)\_s\{-}(\)\@!"
syn match comment "#.*$"
"syn region comments start="/\*" end="\*/"

let b:current_syntax = "yora"

hi def link function	@function
hi def link keyword		@keyword
hi def link string		@string
hi def link character	@character
hi def link comment		@comment
hi def link operator	@operator
hi def link number		@number
hi def link boolean		@boolean
hi def link type		@type
