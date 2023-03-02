pkgname=spaceman
pkgver=git
pkgrel=1
pkgdesc=""
arch=('i686' 'x86_64')
url="https://github.com/salihgerdan/spaceman"
license=('MIT')
depends=(gtk4)
makedepends=(gtk4 cargo)
source=(git+https://github.com/salihgerdan/spaceman.git)
md5sums=("SKIP")

build() {
  cd "$srcdir/$pkgname"
  cargo build --release
}

package() {
  cd "$srcdir/$pkgname"

  mkdir -p "$pkgdir/usr/share/applications"
  install spaceman.desktop "$pkgdir/usr/share/applications"
  mkdir -p "$pkgdir/usr/bin/"
  install target/release/spaceman "$pkgdir/usr/bin/spaceman"
  mkdir -p "$pkgdir/usr/share/pixmaps/"
  install spaceman.png "$pkgdir/usr/share/pixmaps/"
}
