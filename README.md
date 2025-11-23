## nateroids

created to teach [natepiano](https://youtube.com/natepiano) how to code games, visualizations and
simulations in [bevy](https://bevyengine.org) using the awesome programming language, rust. i started
with [this tutorial](https://www.youtube.com/@ZymartuGames),
added [avian3d](https://docs.rs/avian3d/latest/avian3d/) for physics as well as a
few other dependencies you can find in cargo.toml. the goal is to make this interesting, playable, and beautiful.


## first install rust
install rust (from https://www.rust-lang.org/tools/install)

```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## make rust compile faster

use sccache to make follow on compiles faster as it will cache locally anything you've already built. this comes in handy if you get other projects that all need to compile with bevy or 
anything you commonly depend on in these projects

```shell
cargo install sccache
```

and to enable it globally create / edit your $HOME/.cargo/config.toml by adding this to it - make sure you're using your local path - on my system it is:
```shell
[build]
rustc-wrapper = ".cargo/bin/sccache"
```

## clone nateroids project 

```shell
git clone https://github.com/natepiano/nateroids
```

run it - the first time will take a while even if you have sccache installed as you have to populate the cache, n'est-ce pas? This will build a debug version and run it:

```shell
cargo run
```

start playing! 

If you want to run it in release, you can do so by running:

```shell
cargo run --release
```

It might run faster on your machine. It will definitely be a smaller binary.
