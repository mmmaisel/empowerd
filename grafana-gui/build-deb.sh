#!/bin/bash -e

cd "$(dirname "$0")"

npm run build
rm -rf pkg
mkdir -p pkg/DEBIAN

mkdir -p pkg/usr/share/empowerd/grafana
cp -r dist/img pkg/usr/share/empowerd/grafana/
cp dist/module.js pkg/usr/share/empowerd/grafana/
cp dist/plugin.json pkg/usr/share/empowerd/grafana/
cp dist/README.md pkg/usr/share/empowerd/grafana/

VERSION="$(awk < package.json '/version/ { gsub(/[",]/,"",$2); print $2 }')"

cat > pkg/DEBIAN/control <<EOF
Package: empowerd-grafana
Version: ${VERSION}
Architecture: all
Priority: optional
Maintainer: Max Maisel <max.maisel@posteo.de>
Depends: empowerd (>= 0.10.0), grafana (>= 10.0.0)
Description: Empowerd GUI plugin for Grafana
EOF

dpkg-deb --root-owner-group --build pkg
mv pkg.deb "empowerd-grafana_${VERSION}-1_all.deb"
