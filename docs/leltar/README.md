# Funkcióleltár

Gépi, ismételhető leltár arról, mit tud a PicasaPy: ezt kell a PicaRustnak is tudnia a kiadáshoz.
A fájlokat eszköz állítja elő a PicasaPy forrásából; kézzel ne szerkeszd őket.

## Fájlok

- `funkcioleltar.csv`: soronként egy funkció.
- `kulonbseg.md`: ahol a források eltérnek egymástól, és a kézi átnézést kérő sorok.

## A CSV oszlopai

| Oszlop | Jelentés |
|---|---|
| `azonosito` | stabil azonosító a forrásból képezve |
| `terulet` | a felület része vagy a modul, ahová tartozik |
| `tipus` | `menu`, `gomb`, `parbeszed`, `billentyu`, `effekt`, `beallitas` vagy `egyeb` |
| `nev` | a felirat, vagy ha az nem olvasható ki, az elem neve |
| `forras_qml` | a PicasaPy forrásfájlja |
| `forras_teszt` | a PicasaPy tesztjei, amelyek érintik |
| `spec_lap` | a hozzá tartozó leírás a `docs/specs/` alatt, ha megfeleltethető |
| `merfoldko` | melyik belső mérföldkőben készül el (2-6) |
| `allapot` | `nincs`, `folyamatban` vagy `kesz` |

## Mérföldkövek

2 Böngészés, 3 Képnéző és arcok, 4 Szerkesztő, 5 Kimenetek, 6 Kör bezárása.
A besorolás névmintákra épül, ezért tévedhet; a `kulonbseg.md` felsorolja a kézi átnézést kérő sorokat.
