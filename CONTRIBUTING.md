# Közreműködés

A projekt még alakul, nincs kiadás. Hozzájárulás előtt nyiss hibajegyet
(`.github/ISSUE_TEMPLATE/hiba.md`), és írd le, mit szeretnél.

## Szabályok

- Licenc: `GPL-3.0-only`. Új külső csomag csak GPL-3.0-val összeférő licenccel,
  pontosan rögzített verzióval (`=x.y.z`) kerülhet a `[workspace.dependencies]`
  listába; a `Cargo.lock` a repóban van.
- A csomagok közti függések csak a `fuggesek.toml` listájából valók.
- Fájlméret: jellemzően 200-400 sor, legfeljebb 800. Egy változás legfeljebb
  5 fájl és 400 sor, kivéve a gépi másolást.
- Kódkommentek, dokumentáció magyarul. Embernek szóló szövegben nincs nagykötőjel,
  csak sima kötőjel.
- Titok és személyes adat (felhasználónév, saját útvonal, e-mail) soha nem kerül
  a repóba.

## Ellenőrzések

A pull requesten kötelezően fut: `cargo fmt --check`, `cargo clippy -- -D warnings`,
`cargo test`, `cargo deny check`, titokkereső (gitleaks), lefedettségmérés, valamint
a Windowson épített csomag és a szerző-kapu.

## Változásnapló

Minden felhasználót érintő változáshoz tartozik egy töredékfájl a
`changelog.d/<jegyszám>.md` útvonalon, lásd `changelog.d/README.md`.
