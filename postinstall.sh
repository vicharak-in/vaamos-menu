#!/bin/sh

mkdir -p "${DESTDIR}/${MESON_INSTALL_PREFIX}/share/vaamos-menu/"
cp -r "${MESON_SOURCE_ROOT}/src/scripts" "${DESTDIR}/${MESON_INSTALL_PREFIX}/share/vaamos-menu/"
cp -r "${MESON_SOURCE_ROOT}/data" "${DESTDIR}/${MESON_INSTALL_PREFIX}/share/vaamos-menu/"
cp -r "${MESON_SOURCE_ROOT}/ui" "${DESTDIR}/${MESON_INSTALL_PREFIX}/share/vaamos-menu/"

cd "${MESON_SOURCE_ROOT}"/po || exit 0
mkdir -p "${DESTDIR}"/usr/share/locale/en/LC_MESSAGES
#msgfmt -c -o "${DESTDIR}"/usr/share/locale/en/LC_MESSAGES/vaamos-menu.mo en.po
