# neuromaster
3D nmap scan visualizer in Rust, aiming to improve on `zenmap`'s built-in `radialnet`-based visualizer.

Inspired by the descriptions of "The Matrix" in William Gibson's Neuromancer.

Physics-based force-directed-graph methods are used to draw the visualization from the xml.

## usage

neuromaster accepts one optional command line argument, an xml-formatted nmap scan file.
If no command line argument is provided, neuromaster defaults to a built-in scan of `scanme.nmap.org`.

# demo
![nm-scr](https://user-images.githubusercontent.com/77865363/220438442-7f461607-f9c8-476a-a469-cf6ce43daf5e.png)
