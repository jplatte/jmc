# jmc

A simple Matrix client based on
[Druid](https://github.com/linebender/druid).

## Prerequisites

* Install [Rust](https://www.rust-lang.org/tools/install)
    ```sh
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```
* Install the [GTK3](https://www.gtk.org/) development libraries
    ```sh
    sudo apt install libgtk-3-dev
    ```
* Install [clang](https://clang.llvm.org/)
    ```sh
    sudo apt install clang
    ```
* Install [mold](https://github.com/rui314/mold)
    ```sh
    git clone https://github.com/rui314/mold.git
    cd mold
    git checkout v1.0.1
    make -j$(nproc)
    sudo make install
    ```

Alternatively, to avoid the need for `clang` and `mold`, modify
`.cargo/config.toml` to remove the lines mentioning those tools.

## Running

```sh
cargo run
```
