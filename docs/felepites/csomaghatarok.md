# Csomaghatárok (2. mérföldkő)

A munkaterület csomagjai és a köztük engedett függések. A lista gépi párja a gyökér `fuggesek.toml`; amit az nem enged, azt a `crates/pr-core/tests/fuggesek.rs` teszt megfogja. Új él csak az integrátor döntésével kerülhet be.

## Ki mit ad

| Csomag | Mit ad | Mit nem tehet |
|---|---|---|
| `pr-scan` | a képtár mappáinak és képfájljainak listája (`Mappa`, `Foto`), és két bejárás különbsége | nem ír a képtárba; nem függ más belső csomagtól |
| `pr-ini` | a `.picasa.ini` beolvasott tartalma és a visszaírása | más csomag nem írhat `.picasa.ini`-t; nem függ más belső csomagtól |
| `pr-meta` | a képfájl metaadatai (dátum, tájolás, méret, feliratok) | bájtokból dolgozik; nem függ más belső csomagtól |
| `pr-index` | a PicaRust saját SQLite indexe: mappák, képek, keresés | a sémát csak ez a csomag hozza létre és módosítja; a képtárba nem ír |
| `pr-thumbs` | bélyegkép készítése és a gyorsítótár | nem függ a felülettől |
| `pr-faces` | arckeresés és az arcnapló | nem függ a felülettől |
| `pr-render` | effektlánc alkalmazása képre | nem nyúl a lemezhez: képet kap, képet ad; nem függ más belső csomagtól |
| `pr-app` | a Slint-felület és a vezérlők | a felület szála nem végez lemezműveletet, képdekódolást, adatbázis-lekérdezést |

## Engedett élek

- `pr-app` minden más csomagot használhat.
- `pr-index` használhatja: `pr-scan` (a bejárás adatai), `pr-ini` (albumok, csillag, felirat), `pr-meta` (dátum, tájolás).
- `pr-faces` használhatja: `pr-thumbs`.
- A `pr-scan`, a `pr-ini`, a `pr-meta`, a `pr-thumbs` és a `pr-render` nem függ más belső csomagtól.

## A 2. mérföldkő szabálya

A Böngészés csak olvas: ebben a mérföldkőben egyetlen csomag sem ír a képtárba. A `pr-ini` író fele a Szerkesztő mérföldkővel élesedik; addig az oda-vissza egyezést tesztek igazolják, de a program nem hív írást.

## Szálak

Az állapotot háttérmunkás birtokolja; a felület parancsot küld és eredményt rajzol. Lemezolvasás, dekódolás és index-lekérdezés mindig háttérszálon fut, a felület szálán soha.
