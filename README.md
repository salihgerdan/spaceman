# spaceman <img src="spaceman.png" width="48"/>
Treemap disk usage analyzer: *In search of lost space*   
(a.k.a. wata-analyzer)  
**Warning: beta software (however, usable)**
## Features
- [X] Fast scan and display, with the power of Rust, and gtk4 gpu rendering capabilities
- [X] Uses the jwalk library as [dua-cli](https://github.com/Byron/dua-cli/) does, enabling multi-threaded scans
- [x] Visible from the start, incrementally updates the view as the scan goes on
## Planned
- [ ] Tracks changes on the filesystem
- [ ] Navigate into sub-directories
- [ ] Right click menu to manage the directories
## Install
Primarily tested for Linux, but confirmed to work on MacOS and Windows as well.  
**To build**: You need to have `gtk4` (`libgtk-4-dev` on Ubuntu/Debian) and `rust`/`cargo`. The only command necessary to build is `cargo build --release` after the dependencies are installed.  
**To run**: Ensure `gtk4` (`libgtk-4-1` on Ubuntu/Debian) is installed.  
You can also simply grab an executable from the [Releases](https://github.com/salihgerdan/spaceman/releases) section. There is a [PKGBUILD](./PKGBUILD) file available for Arch Linux.  
The Windows package, unlike the Linux and Mac executables, pack all the necessary libraries within the archive so you do not have to install gtk4 separately. I am not quite happy with the Windows package in a UNIX style directory tree, alas this is what was possible with the library dependencies. I am looking into alternatives.  
## Usage
Click the button on the left of the titlebar, and choose a directory to scan. You can also provide a directory to scan as a command line argument.
## Acknowledgements
[SpaceSniffer](http://www.uderzo.it/main_products/space_sniffer/) for inspiration  
[Bruls, Huizing, van Wijk, "Squarified Treemaps"](https://www.win.tue.nl/~vanwijk/stm.pdf) and [TreeMonger](https://github.com/alanbernstein/treemonger) which I referenced for the squarified treemap algorithm
[dua-cli](https://github.com/Byron/dua-cli/) for reference on using the jwalk library  

![Screenshot](screenshot.png?raw=true)
