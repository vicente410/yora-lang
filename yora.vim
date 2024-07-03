syn keyword keyword var if else loop while continue break pr fn return
syn match operator "=\|+\|-\|*\|/\|%\|!\|and\|or\|<\|>"
syn match number '\d\+'
syn keyword boolean true false
syn region string start="\"" end="\""
syn keyword type i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 bool
syn match function "\<\h\w*\ze\_s\{-}(\%(\*\h\w*)\_s\{-}(\)\@!"
syn match comment "#.*$"
"syn region comments start="/\*" end="\*/"

let b:current_syntax = "yora"

hi def link function	@function
hi def link keyword		@keyword
hi def link string		@string
hi def link comment		@comment
hi def link operator	@operator
hi def link number		@number
hi def link boolean		@boolean
hi def link type		@type
