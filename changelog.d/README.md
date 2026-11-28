# Változásnapló-töredékek

Minden felhasználót érintő változáshoz egy töredékfájl tartozik:
`changelog.d/<jegyszám>.md`.

- A fájlban egy vagy több, magyar nyelvű, felhasználói nyelven írt sor van
  (`- ` kezdettel), a végén a jegyszámmal: `(#12)`.
- Belső, a felhasználónak láthatatlan változáshoz (például csak CI) is írj sort,
  ha a projekt működését érinti.
- Kiadáskor az integrátor a töredékeket a `CHANGELOG.md` megfelelő szakaszába
  fűzi, és törli a töredékfájlokat.
- Nagykötőjel és unicode mínusz nem szerepelhet, csak sima kötőjel.
