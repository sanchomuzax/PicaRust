# 0001: Teljes átírás Rustra

- Állapot: elfogadva
- Dátum: 2026-11-27
- Forrás: a tulajdonos által elfogadott terv ([PicaRust: a teljes átírás terve](https://claude.ai/artifact/6ApbC4yCEJdzCCtQqr8erZ)) és a fejlesztési módszertan ([PicaRust: fejlesztési módszertan](https://claude.ai/artifact/VksJU1HC2vtSnYgnYYnc8o))

## Helyzet

A PicasaPy (Python, Qt) a Google Picasa újraírása. A mérések szerint a program keretének lassúsága (indulás, mappanézet akadása, memória) a felületi keretrendszerből ered, nem az effektekből. Ezért a program Rustban készül újra, PicaRust néven.

## A tulajdonos döntései

1. **Rust az irány.** A PicasaPy foltozása helyett új program készül Rustban.
2. **Teljes átírás, részleges kiadás nélkül.** Csonka (például csak nézegetésre alkalmas) kiadás nem megy ki. A PicasaPy marad a kiadott program a PicaRust 1.0-ig, és addig csak hibajavítást kap.
3. **Az eredeti Picasa a mérce.** Az igazság forrása a bináris kutatás eredménye (a `docs/specs/` lapjai). Amit már teljesen felkutattunk és a PicasaPy-ban megírtunk, azt nem kutatjuk újra: ott a PicasaPy rögzített kimenete áll a mérce helyén.
4. **Ugyanazok a kapuk és őrök, mint a PicasaPy-nál.** Érzékeny adat és titok nem kerülhet a repóba.
5. **Előbb a módszertan és a fejlesztési környezet, csak utána a fejlesztés.**
6. **Előbb mérünk, aztán vetünk el egy lehetőséget; a mérés eszköz, nem cél.** A távoli gépen futó munka menet közben látható naplót ír.

## Következmények

- A belső mérföldkövek csak belső előkiadást adhatnak, nyilvános kiadást nem.
- Minden részfeladat a rögzített elváráshoz mérve kerülhet be (`elvarasok/`), a funkcióleltár (`docs/leltar/`) mondja meg, mi tartozik a teljes funkciókészletbe.
- A PicasaPy az átírás alatt nem kap új funkciót.
