# The Felys Programming Language

Felys is a dependency-free interpreted programming language written in Rust, featuring its own compiler and runtime. Feel free to try it out using the online [playground](https://felys.dev/en/compiler) and read the [blog](https://book.felys.dev/purely-to-mention-elysia.html). Please note, however, that the compiler is in a fragile state and requires project-level refactoring to improve code quality.

## About the Agents

This project started in 2023, long before coding agents existed. The [AGENTS.md](AGENTS.md) is here because I asked an LLM to refactor some ugly code. I embrace technology (see my other projects), but I don't like Anthropic having its agent create a C compiler for advertising purposes. It makes people feel as if my language is just some random stuff Claude created from a prompt I gave it. No, it's not. Back in those days, I needed to read texts to really understand how to implement things. That said, if you are learning about compilers, the Claude C Compiler design [document](https://github.com/anthropics/claudes-c-compiler/blob/main/DESIGN_DOC.md) is worth a look.

## References

The following papers, blogs, and books helped me a lot. Also, ask LLMs.

- [Packrat Parsing: Simple, Powerful, Lazy, Linear Time](https://arxiv.org/abs/cs/0603077)
- [PEG Parsing Series Overview](https://medium.com/@gvanrossum_83706/peg-parsing-series-de5d41b2ed60)
- [Simple and Eﬃcient Construction of Static Single
  Assignment Form](https://c9x.me/compile/bib/braun13cc.pdf)
- [Compilers: Principles, Techniques, and Tools](https://en.wikipedia.org/wiki/Compilers:_Principles,_Techniques,_and_Tools)

## License

Distributed under the terms of the [LICENSE](LICENSE).

## Copyright

© All rights reserved by FelysNeko
