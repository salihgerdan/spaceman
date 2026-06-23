# spaceman <img src="spaceman.png" width="28"/>
Treemap disk usage analyzer: *In search of lost space* (a.k.a. wata-analyzer)  
![Screenshot](screenshot.png?raw=true)

## Features
- [x] Uses the jwalk library as the renowned [dua-cli](https://github.com/Byron/dua-cli/) does, enabling multi-threaded fast scans
- [x] Linux-first, cross-platform
- [x] Live display of scan results, responsive UI
- [x] Zoom into sub-directories
- [x] Single portable executable with no dependencies
## Planned
- [ ] More themes
## Install  
**To build**: The only command necessary to build is `cargo build --release`.  
You can also simply grab an executable from the [Releases](https://github.com/salihgerdan/spaceman/releases) section.  
There is an [AUR package `spaceman-git`](https://aur.archlinux.org/packages/spaceman-git) available for Arch Linux (i.e. run `yay install spaceman-git`).  
### Mac tells me this is trash
> "SpaceMan" is damaged and can't be opened. You should move it to the Trash.

As this is an unsigned app, you will have to remove the quarantine after installation with the following command in the terminal.

```sh
xattr -d com.apple.quarantine /Applications/SpaceMan.app
```
## Usage
Click the button on the left of the titlebar, and choose a directory to scan. You can also provide a directory to scan as a command line argument.

Don't forget to empty the system trash after using the "trash" option in order to reclaim the space.
## News! (exciting)
Version 0.2.0 was a near-rewrite of the app and we switched from using GTK4 to Iced as our UI library. As I wanted to make this a portable application, this finally allowed us to avoid shipping .DLL files or asking to install dependencies.
## Acknowledgements
[SpaceSniffer](http://www.uderzo.it/main_products/space_sniffer/) for inspiration  
[Bruls, Huizing, van Wijk, "Squarified Treemaps"](https://www.win.tue.nl/~vanwijk/stm.pdf) and [TreeMonger](https://github.com/alanbernstein/treemonger) which I referenced for the squarified treemap algorithm  
[dua-cli](https://github.com/Byron/dua-cli/) for reference on using the jwalk library
