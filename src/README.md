> [!NOTE]
> This file is used to generate [the Typst Universe page](https://typst.app/universe/package/board-n-pieces). It is processed by [`/build.py`](/build.py).


# Xodec

This package makes it possible to find the names of a symbol in [Codex](https://github.com/typst/codex), the library that populates the `sym` and `emoji` modules in Typst.


## Usage

```example
#get-names("x") \
#get-names("∅") \
#get-names("➡\u{FE0E}") \
#get-math-names("4") \
#get-math-names("ϕ") \
#get-math-names("ℒ\u{FE00}") \
#get-math-names("ℕ") \
#get-math-names("𝑀") \
#get-math-names("≠")
```
