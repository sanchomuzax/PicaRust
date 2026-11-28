# Kihagyott elvárás-adatok

Ami nem került át, annak az okával. Az átvétel semmit nem talál ki: ami eredetét vagy licencét a PicasaPy nem írja le, az itt áll.

<!-- szakasz:effektek -->
## Effektek

- `crop`: történeti mód (vágás), nem képszámító effekt
- `crop64`: történeti mód (vágás-előzmény), nem képszámító effekt
- `debug`: hibakereső bejegyzés, nincs képszámító feldolgozója
- `focalpixelate (alap, `focalpixelate=1,0.5,0.5,15,1,0.25,0;`)`: szinatmenet: a PicasaPy láncában nincs feldolgozója (nincs képszámító modell)
- `focalpixelate (alap, `focalpixelate=1;`)`: szinatmenet: a PicasaPy láncában nincs feldolgozója (nincs képszámító modell)
- `focalpixelate (nem_alap, `focalpixelate=1,0.5,0.5,57.5,1.5,0.6,0.5;`)`: szinatmenet: a PicasaPy láncában nincs feldolgozója (nincs képszámító modell)
- `linblur (nem_alap, `linblur=1,0.5,0.5,6;`)`: a nem alapértelmezett készlet nem változtatta meg a kimenetet az alapértelmezetthez képest
- `movieend`: filmkészítő jelző, nincs képszámító feldolgozója
- `moviestart`: filmkészítő jelző, nincs képszámító feldolgozója
- `picnik`: történeti mód (külső szerkesztő jelzője), nem képszámító effekt
- `picnikfocalpixelate (nem_alap, `PicnikFocalPixelate=1,0.5,0.5,60,0,75,50;`)`: a nem alapértelmezett készlet nem változtatta meg a kimenetet az alapértelmezetthez képest
- `radsat (nem_alap, `radsat=1,0.5,0.5,0.5,0.5;`)`: a nem alapértelmezett készlet nem változtatta meg a kimenetet az alapértelmezetthez képest
- `radtint (nem_alap, `radtint=1,0.5,0.5,0.625;`)`: a nem alapértelmezett készlet nem változtatta meg a kimenetet az alapértelmezetthez képest
- `reanimatedeyecolor (nem_alap, `ReanimatedEyeColor=1,18,60;`)`: a nem alapértelmezett készlet nem változtatta meg a kimenetet az alapértelmezetthez képest
- `redeye`: történeti mód, régió-adatot kér (a vörösszem javítása)
- `retouch`: történeti mód, régió-adatot kér (retusálás)
- `rot`: történeti mód (forgatás-jelző), nem képszámító effekt
- `save`: történeti mód (mentési jelző), nem képszámító effekt
<!-- /szakasz:effektek -->
<!-- szakasz:mintak -->
## Mintafájlok

- nem adat fajtájú teszt használja (csak felületi referenciaképként szerepel) (14 fájl):
  - `tests/fixtures/face_type_a_name_reference_en38.png`
  - `tests/fixtures/people_panel_named_row_4522.png`
  - `tests/fixtures/slideshow_6102/controls-595.png`
  - `tests/fixtures/slideshow_6102/controls-600.png`
  - `tests/fixtures/slideshow_6102/controls-605.png`
  - `tests/fixtures/slideshow_6102/image-595.png`
  - `tests/fixtures/slideshow_6102/image-600.png`
  - `tests/fixtures/slideshow_6102/image-605.png`
  - `tests/fixtures/slideshow_6102/paused-595.png`
  - `tests/fixtures/slideshow_6102/paused-600.png`
  - `tests/fixtures/slideshow_6102/paused-605.png`
  - `tests/fixtures/slideshow_6102/transition-end-595.png`
  - `tests/fixtures/slideshow_6102/transition-end-600.png`
  - `tests/fixtures/slideshow_6102/transition-end-605.png`
- tesztkód, nem mintafájl (6 fájl):
  - `tests/golden/__init__.py`
  - `tests/golden/test_compare_render.py`
  - `tests/golden/test_keszlet_jelzes_1620.py`
  - `tests/golden/test_make_golden_kit_effects.py`
  - `tests/golden/test_make_param_sweep.py`
  - `tests/golden/test_synthetic_photos_noise.py`
<!-- /szakasz:mintak -->
