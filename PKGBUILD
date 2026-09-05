pkgname=feedmee
pkgver=0.22.11
pkgrel=1
pkgdesc="The seriously fast feed reader."
arch=('x86_64')
license=('MIT')
options=('!debug')
depends=('webkit2gtk-4.1' 'gtk3')
source=()
sha256sums=()

# Binary is already built locally; this only packages it and the support files.
package() {
  install -Dm755 "$startdir/../src-tauri/target/release/FeedMee" "$pkgdir/usr/bin/feedmee"

  install -Dm644 "$startdir/../feedmee.desktop" "$pkgdir/usr/share/applications/feedmee.desktop"

  install -Dm644 "$startdir/../src-tauri/icons/icon.png" "$pkgdir/usr/share/icons/hicolor/512x512/apps/feedmee.png"
  install -Dm644 "$startdir/../src-tauri/icons/128x128.png" "$pkgdir/usr/share/icons/hicolor/128x128/apps/feedmee.png"
  install -Dm644 "$startdir/../src-tauri/icons/64x64.png" "$pkgdir/usr/share/icons/hicolor/64x64/apps/feedmee.png"
  install -Dm644 "$startdir/../src-tauri/icons/32x32.png" "$pkgdir/usr/share/icons/hicolor/32x32/apps/feedmee.png"
}