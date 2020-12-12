# Ricochet Robots Solver

A collection of crates for solving the board game ricochet robots.

## Usage

Use `cargo run` to use the cli tool and solve a game.
You will first have to select the board quarters which make up the board, starting from the upper left and rotating clockwise. They are assigned colors depending on their actual board game counterparts.

See [Installation](#installation) for getting cargo.

## Structure

This project is split into three parts:

- `ricochet_board` contains the implementation of the board and game rules.
- `ricochet_solver` contains everything concerning solvers.
- `ricli` is a cli tool to use the other parts.

## Installation

Building from source requires a stable rust compiler which can be installed using [rustup](https://rustup.rs/).

## Board editor

Board editor written in elm hosted [here](https://lireer.github.io/ricochet-robot-solver/), but right now it's not really usable for anything other than moving pieces around.
