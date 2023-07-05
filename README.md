# Trilogy

[![Spec](https://github.com/foxfriends/trilogy/actions/workflows/spec.yaml/badge.svg)](https://github.com/foxfriends/trilogy/actions/workflows/spec.yaml)
[![Rust CI](https://github.com/foxfriends/trilogy/actions/workflows/rust.yml/badge.svg)](https://github.com/foxfriends/trilogy/actions/workflows/rust.yml)

My (approximately) third attempt at building a programming language[^1][^2].

Also a programming language that more closely resembles three languages
at once than any one single language.

Also an exploration into a bunch of ideas in language theory (or at least my own
ideas of them) that I am finding come in sets of three.

That makes a trilogy. Hopefully I produce something useful. Third time's the charm!

[^1]: First few were school projects, WLP4 (a subset of C), and Joos (a subset of Java), and maybe a few others. Though technically I did (attempt to) implement them (with friends), they aren't *my* languages, so I cannot provide the source code.
[^2]: The first I could call my own was [Lumber](https://github.com/foxfriends/lumber), an experiment at a functioning language.

## Status

As you can see, progress at this time is limited:

- [x] Read some papers and instructions on programming languages
    - https://www.eff-lang.org/handlers-tutorial.pdf
    - http://www.math.bas.bg/bantchev/place/iswim/j-explanation.pdf
    - http://www.math.bas.bg/bantchev/place/iswim/j.pdf
    - https://cs.ru.nl/~dfrumin/notes/delim.html
    - https://www.cs.cmu.edu/~rwh/students/filinski.pdf
    - https://homepages.inf.ed.ac.uk/slindley/papers/effmondel-jfp.pdf
    - https://caml.inria.fr/pub/papers/xleroy-applicative_functors-popl95.pdf
    - https://doc.rust-lang.org/reference/
    - https://github.com/HigherOrderCO/HVM
    - https://www.sciencedirect.com/science/article/pii/S0890540197926432/pdf?md5=30965cec6dd7605a865bbec4076f65e4&pid=1-s2.0-S0890540197926432-main.pdf
- [x] Design the language: Check out the [spec](./spec/)!
- [x] Read the [book](https://craftinginterpreters.com/)
- [ ] Specify the language (In progress)
- [x] Start the project:
    - [x] Scanning
    - [x] Parsing
    - [x] Syntactic analysis
    - [ ] Name resolution (In progress)
    - [ ] Bytecode generation
    - [x] Virtual machine
    - [ ] Testing (In progress)
    - [ ] Optimization
    - [ ] Standard library
