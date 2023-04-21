{ pkgs ? import <nixpkgs> {} }:
with pkgs;
mkShell {
  buildInputs = [ gcc pkg-config gdk-pixbuf gtk3 webkitgtk ];
}
