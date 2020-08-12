//! A module for playing and mixing GBA audio, providing playback and mixer functionality.
//! Based on [this tutorial](https://web.archive.org/web/20150828041649/http://deku.rydia.net/program/sound1.html)
//! and [this one](http://archive.gamedev.net/archive/reference/programming/features/gbasound1/index.html).

mod player;
pub use player::Player;
#[cfg(test)]
mod test;
mod wave;
