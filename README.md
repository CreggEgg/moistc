# Moistc
Moistc is a joke programming language made for my own entertainment. Please do not use it under any circumstances for anything more than novelty.
---
## Setup
- Run this code in your terminal: `cargo install --git https://github.com/creggegg/moistc`
- Install a c compiler (I use gcc)
- Download the core.c file from this github repository and put in in your project folder
- Create a file with the extension .wet
- Write your code
- Run `moistc build <filename>.wet && gcc main.o core.c`
- This will produce an executable! YAY!

## Examples
### Hello world
```
funion main[] (
  println["hello world"]
)
```
This is the classic hello world function. the main function is the entry point of all moistc programs.
### Variables & functions
```
funion add[a: Int, b: Int] (
  (sum is a + b).
  sum;
)
funion main[] (
  printintln[add[5,6]]
)
```
This is a simple example of functions and variable declarations. Variable declarations are "(<ident> is <value>)" and must be contained in parenthesees because the parser is not very good. The period operator discards the value of the lhs and uses the value of the rhs and is ended with a semicolon.
### Period operator
```
funion main[] (
  printintln[5].
  printintln[6].
  printintln[7]
  ;;
)
```
The period operator can be chained as shown above. This results in a large amount of semicolons at the end of the expression. I like having this on a new line but it doesn't really matter you could do it however you like because this is objectively stupid (parser bad moment).
