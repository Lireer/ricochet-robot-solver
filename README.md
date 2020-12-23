# Ricochet Robots Solver

A collection of crates for solving the board game ricochet robots.

## Usage

Use `cargo run --release` to use the cli tool and solve a game.
You will first have to select the board quarters which make up the board, starting from the upper left and rotating clockwise. They are assigned colors depending on their actual board game counterparts.

See [Installation](#installation) for getting cargo.

## Structure

This project is split into three parts:

- `ricochet_board` contains the implementation of the board and game rules.
- `ricochet_solver` contains everything concerning solvers.
- `ricli` is a cli tool to use the other parts.

## Building from source

**Rust**: Building from source requires a stable rust compiler which can be installed using [rustup](https://rustup.rs/).
    If no python interop is needed, the rust code can be compiled with `cargo build --release` or run with `cargo run --release`.

**Python**: Using a virtual environment is recommended, e.g. conda. To create python packages from rust code, install [maturin](https://pypi.org/project/maturin/) in the environment.

```bash
pip install maturin
```

Navigate to the rust package that is to be built. Use `maturin develop` to build and install it in the environment.

## Board editor

Board editor written in elm hosted [here](https://lireer.github.io/ricochet-robot-solver/), but right now it's not really usable for anything other than moving pieces around.
