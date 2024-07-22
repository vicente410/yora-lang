# Language Tour
This is a tour of all the features currently present on yora 0.1.0. For further examples check the code in this directory.

## Hello World
Programs in Yora are read from top to bottom. You can start writting right away, no need for a main procedure.
```nim
print("Hello World\n")
```

## Variables
You can declare variables with the var keyword.
```nim
var answer = 42
```

Yora is still strongly and statically typed, that is, the type of a variable is fixed. The interpreter will try to infer the type of the variable from context. You can also specify the type by giving a hint.
```nim
var hinted_var: Int = 4
```

Variables, as the name implies, can change.
```nim
var answer = 3
print(answer) # 3

answer = 42
print(answer) # 42
```

You may also use arithmetric operators with an assignment.
```nim
var count = 0
count += 1
```

## Types
There are three primitive types. Type hints are given for clarity, but remember that they are not needed.
```nim
var integer: Int = 20
var boolean: Bool = true
var character: Char = 'B'
```

There is also the array type, wich can be made by appending "[]" to a type.
```nim
var nums: Int[] = [1, 2, 3]
```

You may use a string literal to specify a Char[].
```nim
var name: Char[] = "Sophie"
```

Arrays can be indexed, returning the element at that position.
```nim
var nums = [1, 2, 3]
print(nums[0]) # 1

var planet = "Earth"
print(planet[2]) # r
```

## Operators
### Arithmetric
Operators that recieve two Int's and return an Int.
```nim
print(3 + 5) # 8
print(3 - 5) # -2
print(3 * 5) # 15
print(3 / 5) # 0
print(3 % 5) # 3
```

### Comparison
Operators that recieve two Int's and return a Bool.
```nim
print(3 != 5) # true
print(3 == 5) # false
print(3 < 5) # true
print(3 <= 5) # true
print(3 > 5) # false
print(3 >= 5) # false
```

### Boolean Algebra
Operators that recieve Bool's and return a Bool
```nim
print(true and false) # false
print(true or false) # true
print(!false) # true
```

## Control Flow
Control flow structures are defined by their indentation. Every line that is to the front of the structure is inside it.
### If
The block is only ran if the condition is true.
```nim
if condition():
    do_stuff() # this is inside the if

do_other_stuff() # this is outside the if
```

An else block is only ran if the condition is false.
```nim
if condition():
    do_stuff()
else:
    do_other_stuff()
```

Lastly, you can also chain "if else"'s.
```nim
if condition1():
    do_stuff1()
else if condition2():
    do_stuff2()
else if condition3():
    do_stuff3()
else:
    do_stuff4()
```

### While
Checks the condition at the beggining of each iteration and repeats while it is true.
```nim
var num = 0
while num < 10:
    num += 1
```

A continue will skip to the next iteration in the loop.
```nim
while condition():
    var num = string_to_int(input())
    if num >= 10:
        continue
    do_stuff()
```

A break ends the loop.
```nim
var num = 0
while condition():
    print(num)
    num += 1
    if num >= 10:
        break
```

### Loop
You can make a loop without having to have a while true.
```nim
loop:
    var str = input()
    print(str)
```
Breaks and continues can also be used in whiles.

## IO
### Input
You can read input from the user with the input() procedure. It returns a new Char[] with the input given. It can than be turned into an Int with string\_to\_int().
```nim
var str = input()
print(str)

var num = string_to_int(input())
print(num + 5)
```

### Output
You can print a char array or any primitive type directly.
```nim
print(42) # 42
print(true) # true
print('B') # B
print("Answer") # Answer
```

## Procedures
Procedures are the building blocks of code. A procedure can have any number of inputs and it will run its block with the given inputs. It may also have an output which can be returned with the return keyword. Procedures must be declared before usage. Notice that all inputs are passed by value, not by reference.
```nim
pr is_multiple_of_two(num: Int) -> Bool: # a return type can be specified with an arrow
    return num % 2 == 0

print(is_multiple_of_two(7))
```
