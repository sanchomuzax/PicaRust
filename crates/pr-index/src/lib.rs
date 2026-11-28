//! A PicaRust saját indexe.
//!
//! Az SQLite sémát kizárólag ez a csomag hozza létre és módosítja; a többi csomag csak a nyilvános
//! lekérdezéseit hívja. Az index bármikor újraépíthető a képtárból, a képtárba nem ír.
#![forbid(unsafe_code)]
