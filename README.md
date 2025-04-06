## Synopsis

This repository contains the source code of the book ["Programming a toy computer from scratch"](https://ebruneton.github.io/toypc/toypc.pdf), and of the associated ["website"](https://ebruneton.github.io/toypc/) and ["ToyPC emulator"](https://ebruneton.github.io/toypc/emulator.html). 

## Demo

Use the online ["demo"](https://ebruneton.github.io/toypc/emulator.html?script=backups/final.txt) of the fully assembled and programmed toy computer to see what it can do. For instance, type `snake` to launch its snake game, or `edit src/snake/snake.toy` to view the source code of this program.

## Build

Type `make` in the `book` directory to build the book. This requires [GNU Make](https://www.gnu.org/software/make/), [Python3](https://www.python.org/downloads/), a complete [LaTeX](https://www.latex-project.org/get/) installation, and a complete [Rust](https://www.rust-lang.org/tools/install) installation (including Cargo).

Type `make` in the main directory to build the book and the companion website as well.

## License

The LaTeX source code of the book is licensed under the [Creative Commons BY-NC-SA 4.0
License](https://creativecommons.org/licenses/by-nc-sa/4.0/). The custom LaTeX source code preprocessor (used for literate programming), as well as the toy PC emulator (used to check the book content), both in Rust, are licensed under the [GNU
General Public License v3](https://www.gnu.org/licenses/gpl-3.0.en.html).

