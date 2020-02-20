pub(crate) struct Map<'a> {
    tileset: &'a [u32], // Tileset to display
    palette: &'a [u32], // Palette for tiles
    tilemap: &'a [u32], // Tilemap to display
}

impl Map<'_> {
    pub(crate) fn new_empty() -> Map<'static> {
        return Map {
            tileset: &[0],
            tilemap: &[0],
            palette: &[0],
        };
    }
    pub(crate) fn draw(&self) {}
}
