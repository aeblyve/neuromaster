# neuromaster
3D nmap scan visualizer in Rust, aiming to improve on `zenmap`'s built-in `radialnet`-based visualizer, both in graphics performance (for `radialnet` is cpu-rendered) and topology comprehension (for `radialnet` struggles to fit many nodes on screen).

Inspired by the descriptions of "The Matrix" in William Gibson's novel _Neuromancer_.

Physics-based force-directed-graph methods are used to draw the visualization from the xml.

## usage

neuromaster accepts one optional command line argument, an xml-formatted nmap scan file.
If no command line argument is provided, neuromaster defaults to a built-in scan of `scanme.nmap.org`.

## building

`neuromaster` requires rust nightly to build, which is specified in `rust-toolchain.toml`. `cargo run` should just work on most systems.
On NixOS, a `flake.nix` is provided to build the executable with `nix build`.

The application has been tested as working on Arch Linux and NixOS. Other operating systems have not been tested.


# demo
![nm-scr](https://user-images.githubusercontent.com/77865363/220438442-7f461607-f9c8-476a-a469-cf6ce43daf5e.png)
