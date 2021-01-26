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
- `ricochet_env` contains a rust RL environment and its python bindings.
- `ricli` is a cli tool to use the other parts.

## Building from source

**Rust**: Building from source requires a stable rust compiler which can be installed using [rustup](https://rustup.rs/).
    If no python interop is needed, the rust code can be compiled with `cargo build --release` or run with `cargo run --release`.

**Python**: The following is only necessary if you plan to build the python RL environment.

At least Python 3.6 is required (only tested with 3.8) and a virtual environment has to be used, e.g. conda. To create python packages from rust code, install [maturin](https://pypi.org/project/maturin/) in the environment.

```bash
pip install maturin
```

To build the environment either call `maturin develop --release` in the `ricochet_env` directory or from the project root call

```bash
$ maturin develop --release --manifest-path ricochet_environment/Cargo.toml 
🍹 Building a mixed python/rust project
🔗 Found pyo3 bindings
🐍 Found CPython 3.8 at python3
    Finished release [optimized] target(s) in 0.03s
```

`maturin build` can be used instead to build but not install the package. The built `.whl` file can be found in `target/wheels/` and can be installed using pip.

Navigate to the rust package that is to be build. Use `maturin develop --release` to build and install it in the environment.

## Using the reinforcement learning environment

The `ricochet_env` package has to be installed in the virtual environment, see [Building from source](#building-from-source) for installation instructions. The package can be used like this:

```python
from ricochet_env import RicochetEnv

env = RicochetEnv()
env.step(0)
env.step(1)
(observation, reward, done, info) = env.step(2)
assert(reward == 1.0)

env.reset()
(observation, reward, done, info) = env.step(0)
assert(reward == 0.0)
```

## Board editor

Board editor written in elm hosted [here](https://lireer.github.io/ricochet-robot-solver/), but it's not really usable for anything other than moving pieces around.
