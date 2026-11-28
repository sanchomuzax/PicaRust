# Rögzített elvárások

A PicaRust kimenetét ehhez mérjük. A fájlokat eszköz állította elő a PicasaPy-ból; kézzel ne szerkeszd őket.
Ez a mappa védett: a munkások nem módosíthatják, változtatni csak az integrátor tud.

- `JEGYZEK.json`: minden fájl útvonala, sha256-kivonata, eredete (a PicasaPy melyik állapotából készült) és licence.
- `KIHAGYVA.md`: ami nem került át, és miért.
- `effekt/`: effekt-referenciák. A bemenet már dekódolt, veszteségmentes kép (PNG, 8 bit RGB), programmal előállítva; a kimenet is veszteségmentes. Így a képek beolvasásának eltérése nem keveredik az effekt eltérésébe.
- `egyeb/`: a PicasaPy adattesztjeinek mintafájljai.

A küszöb alapból nulla: a Rust-kimenetnek képpontra egyeznie kell. Kivételt csak az integrátor rögzíthet, méréssel.
