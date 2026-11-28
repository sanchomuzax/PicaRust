//! A képtár bejárása és a változások észlelése.
//!
//! Ez a csomag olvassa a lemezről a mappák és a képfájlok listáját, és megmondja, mi változott két
//! bejárás között. A képtárba nem ír. A felülettől nem függ.
#![forbid(unsafe_code)]
