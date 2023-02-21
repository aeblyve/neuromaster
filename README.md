# neuromaster
3D nmap scan visualizer in Rust, aiming to improve on `zenmap`'s built-in `radialnet`-based visualizer.

Inspired by the descriptions of "The Matrix" in William Gibson's Neuromancer.

## usage

neuromaster accepts one optional command line argument, an xml-formatted nmap scan file.
If no command line argument is provided, neuromaster defaults to a built-in scan of `scanme.nmap.org`.
