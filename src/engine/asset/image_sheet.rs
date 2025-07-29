use crate::engine::geometry::Size2D;


#[derive(Hash, PartialEq, Eq, Debug, Clone)]
pub(crate) struct ImageSheetAsset {
    pub(crate) path: String,
    pub(crate) tile_size: Size2D
}

impl ImageSheetAsset {
    pub(crate) fn new(path: &str, tile_size: Size2D) -> Self {
        Self {
            path: String::from(path),
            tile_size
        }
    }
}