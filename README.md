# PicaRust

A [PicasaPy](https://github.com/sanchomuzax/PicasaPy) fotókezelő teljes átírása Rustra.
A Picasa korábbi, Google-féle fotókezelő és képszerkesztő programjának újraalkotása,
GPL-3.0 licenc alatt.

## Állapot

**Fejlesztés alatt, nincs kiadás.** Még nincs használható program, és nyilvános
kiadás sem lesz a teljes funkciókészletig. A kiadott, használható program ma a
[PicasaPy](https://github.com/sanchomuzax/PicasaPy): ha fotókezelőt keresel, azt
telepítsd.

Ebben a szakaszban csak a munkaterület és az automatikus ellenőrzés áll
(`crates/pr-core`, CI). Az elvárt viselkedések leírása az `elvarasok/`, a
funkcióleltár a `docs/leltar/` mappában van.

## Felépítés

- `crates/`: a munkaterület csomagjai (`pr-*`), egy csomag egy felelősség.
- `fuggesek.toml`: a csomagok közt engedett függések listája; a `pr-core` tesztje őrzi.
- `deny.toml`: licenc-, biztonsági, tiltási és forrásszabályok (`cargo-deny`).
- `changelog.d/`: a változásnapló töredékei (lásd `CONTRIBUTING.md`).

## Licenc

GPL-3.0-only, lásd a `LICENSE` fájlt.
