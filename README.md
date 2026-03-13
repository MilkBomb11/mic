# mic

A compiled toy language similar to C. It can handle mutual recursion and nested functions.

## Installation
1. Install gcc or clang.
2. Download the QBE 1.2 compilier backend [here](https://c9x.me/compile/releases.html.) and install it with the Makefile.
3. Download the files from [releases](https://github.com/MilkBomb11/mic/releases). 

## Build from Source
1. Install gcc or clang.
2. Download the QBE 1.2 compilier backend [here](https://c9x.me/compile/releases.html.) and install it with the Makefile.
3. Download [rust](https://rust-lang.org/).
4. Clone this repo to your local computer via `git clone https://github.com/MilkBomb11/mic.git`
5. Do `cargo build` on the cloned repo. 
6. the executable is `target/debug/mic`

## Usage
```
/path/to/mic [-l] </path/to/your/src/file> </path/to/your/dest/file>
    -l : turn it on if you like logs.  
```

## Snippets
A simple hello world program.
___
```
byte s[15];

s[0] = 'H';
s[1] = 'e';
s[2] = 'l';
s[3] = 'l';
s[4] = 'o';
s[5] = ',';
s[6] = ' ';
s[7] = 'W';
s[8] = 'o';
s[9] = 'r';
s[10] = 'l';
s[11] = 'd';
s[12] = '!';
s[13] = '\n';
s[14] = '\0';

print_string(s);
```

Nested functions.
___
```
int global_offset = 100;

int math_helper(int a, int b) {
    return (a * b) - (a / 2);
}

int closure_test(int x) {
    int secret_val = 42;

    int inner1() {
        int inner2() {
            return secret_val + global_offset; 
        }
        
        return inner2() + 1;
    }

    return inner1() + math_helper(x, 4);
}

int result = closure_test(10); 

print_int(result);

print_byte(10 as byte);
```

Pointers.
___
```
int arr1[3];
int arr2[3];

int i = 0;
while i < 3 {
    arr1[i] = i; 
    i = i + 1;
}

i = 0;
while i < 3 {
    arr2[i] = i+3; 
    i = i + 1;
}

int main()
{
    ptr<int> arr[2];
    arr[0] = arr1 as ptr<int>;
    arr[1] = arr2 as ptr<int>;

    ptr<ptr<int>> p = (&arr[0]) as ptr<ptr<int>>;
    p[0][1] = 42;
    p[1][2] = 10;
    
    int i = 0;
    while (i < 2) {
        int j = 0;
        while (j < 3) {
            print_int(*(*(p+i) + j));
            print_byte(' ');
            j = j + 1;
        }
        i = i + 1;
        print_byte('\n');
    }

    return 0;
}

main();
```

Fibonacci sequence.
___
```
int fib(int n) {
    if n <= 1 { return n; }
    return fib(n-1) + fib(n-2);
}

int i = 0;
while i < 10 {
    print_int(fib(i));
    print_byte(' ');
    i = i + 1;
}
print_byte('\n');
```

Input a radius and print a circle.
___
```
byte matrix[51][51];

int r;
get_int (&r);

int i = 0;
while i < 51 {
    int j = 0;
    while j < 51 {
        if (i - 51/2)*(i - 51/2) + (j - 51/2) * (j - 51/2) <= r*r {
            matrix[i][j] = '*';
        }
        else {matrix[i][j] = ' ';}
        j = j + 1;
    }
    i = i + 1;
}

i = 0;
while i < 51 {
    int j = 0;
    while j < 51 {
        print_byte(matrix[i][j]);
        j = j + 1;
    }
    print_byte('\n');
    i = i + 1;
}
```

Mutual recursion to check if a number is even or odd. (Very inefficient, I know.)
___
```
int x;

int main() {
    bool is_odd(int x) { 
        if x == 0 {return false;}
        if x == 1 {return true;}
        return is_even(x-1);    
    }

    bool is_even(int x) {
        if x == 0 {return true;}
        if x == 1 {return false;}
        return is_odd(x-1);
    }
    
    get_int(&x);
    print_int(is_even(x) as int);
    print_byte('\n');

    return 0;
}

main();
```

Echo.
___
```
byte s[50];
byte c = '\x00';
int idx = 0;
while idx < 49 && c != '\n' {
    get_byte(&c);
    s[idx] = c;
    idx = idx + 1;
}
s[idx] = '\x00';

print_string(s);
```

Sierpinski's triangle.
___
```
byte canvas[100][100];

int sierpinski(int si, int sj, int size) {
    if (size == 1) {
        canvas[si][sj] = '*';
        return 0;
    }
    
    int half = size / 2;
    
    sierpinski(si, sj, half);                 // Top-Left
    sierpinski(si, sj + half, half);          // Top-Right
    sierpinski(si + half, sj + half, half);   // Bottom-Right
    
    return 0;
}

int n;
get_int(&n); 

int i = 0;
while i < n {
    int j = 0;
    while j < n {
        canvas[i][j] = ' ';
        j = j + 1;
    }
    i = i + 1;
}

sierpinski(0, 0, n);

i = 0;
while i < n {
    int j = 0;
    while j < n {
        print_byte(canvas[i][j]);
        j = j + 1;
    }
    print_byte('\n');
    i = i + 1;
}
```

## BNF
```
<program>        ::= <stmt>*

<block>          ::= "{" <stmt>* "}"

<stmt>           ::= <type> <ident> <array_dims>? ";"                // Declaration
                   | <type> <ident> "=" <expr> ";"                   // Definition
                   | <type> <ident> "(" <param_list>? ")" <block>    // Function declaration
                   | <expr> "=" <expr> ";"                           // Assignment
                   | <expr> ";"                                      // Expression stmt
                   | "if" <expr> <block> ( "else" <stmt> )?
                   | "while" <expr> <block>
                   | "return" <expr> ";"
                   | "break" ";" 
                   | "continue" ";"
                   | "print_int" <expr> ";"
                   | "print_byte" <expr> ";"
                   | "print_string" <expr> ";"
                   | "get_int" <expr> ";"
                   | "get_byte" <expr> ";"
                   | <block>

<array_dims>     ::= ( "[" <number> "]" )+
    
<expr>           ::= <logical_or>

<logical_or>     ::= <logical_and> ( "||" <logical_and> )*
<logical_and>    ::= <equality> ( "&&" <equality> )*
<equality>       ::= <relational> ( ( "==" | "!=" ) <relational> )*
<relational>     ::= <additive> ( ( "<" | "<=" | ">" | ">=" ) <additive> )*
<additive>       ::= <mult> ( ( "+" | "-" ) <mult> )*
<mult>           ::= <unary> ( ( "*" | "/" ) <unary> )*

<unary>          ::= ( "+" | "-" | "!" | "&" | "*" ) <unary>
                   | <postfix>

<postfix>        ::= <primary> ( "[" <expr> "]" )*
                   | <ident> "(" <arg_list>? ")" // Function call
                   | <primary> ( "as" <type> )*          

<primary>        ::= <ident>
                   | <int_lit>
                   | <byte_lit>                                 
                   | "true" | "false"
                   | "(" <expr> ")"

<arg_list>       ::= <expr> ( "," <expr> )*

<param_list>     ::= <param> ( "," <param> )*
<param>          ::= <type> <ident>

<type>           ::= "int" 
                   | "byte" 
                   | "bool"
                   | "ptr" "<" <type> ">"

<ident>          ::= ( <letter> | "_" ) ( <letter> | <digit> | "_" )*
<letter>         ::= [a-z] | [A-Z]
<digit>          ::= [0-9]

<int_lit>        ::= "0" 
                   | <non_zero_digit> <digit>*

<non_zero_digit> ::= [1-9]
<byte_lit>       ::= "'" <raw_char> "'" 
                   | "'" "\" <escape_char> "'"
                   | "'" "\" "x" <hex_digit> <hex_digit> "'"

<raw_char>       ::= /* Any printable ASCII character except ' and \ */
<escape_char>    ::= "n" | "r" | "t" | "0" | "\" | "'" | '"'
<hex_digit>      ::= [0-9] | [a-f] | [A-F]
```