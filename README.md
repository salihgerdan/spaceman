# spaceman <img src="spaceman.png" width="48"/>
Treemap disk usage analyzer: In search of lost space   
(a.k.a. wata-analyzer)  
**Warning: beta software**
## Features
- [X] Fast scan and display, with the power of Rust, and gtk4 gpu rendering capabilities
- [X] Uses the jwalk library as [dua-cli](https://github.com/Byron/dua-cli/) does
- [x] Visible from the start, updates as the scan goes on
- [X] Tight and informative presentation of data
## Planned
- [ ] Tracks changes on the filesystem
- [ ] Navigate into sub-directories
- [ ] Right click menu to manage the directories
## Install
Only tested on Linux. You need to have `gtk4` and `rust` installed in your system to build, and only `gtk4` to run it. You can simply grab an executable from the Releases section.
There is a [PKGBUILD](./PKGBUILD) file available for Arch Linux.
## Usage
Click the button on the left of the titlebar, and choose a directory to scan.
## Acknowledgements
[SpaceSniffer](http://www.uderzo.it/main_products/space_sniffer/) for inspiration  
[Bruls, Huizing, van Wijk, "Squarified Treemaps"](https://www.win.tue.nl/~vanwijk/stm.pdf) and [TreeMonger](https://github.com/alanbernstein/treemonger) which I referenced for the squarified treemap algorithm

![Screenshot](screenshot.png?raw=true)
