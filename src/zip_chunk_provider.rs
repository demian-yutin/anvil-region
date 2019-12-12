use log::debug;
use nbt::CompoundTag;
use zip::ZipArchive;
use std::io::{Cursor, Read, Seek};
use crate::{AnvilChunkProvider, AnvilRegion, ChunkSaveError, ChunkLoadError, RegionAndOffset};

/// The chunks are read from a zip file
pub struct ZipChunkProvider<R: Read + Seek> {
    zip_archive: ZipArchive<R>,
}

impl<R: Read + Seek> ZipChunkProvider<R> {
    pub fn new(reader: R) -> Self {
        let mut zip_archive = ZipArchive::new(reader).unwrap();
        debug!("Contents of zip archive:");
        for i in 0..zip_archive.len() {
            let file = zip_archive.by_index(i).unwrap();
            debug!("Filename: {}", file.name());
        }

        ZipChunkProvider { zip_archive }
    }
    pub fn region_path(region_x: i32, region_z: i32) -> String {
        format!("region/r.{}.{}.mca", region_x, region_z)
    }
}

impl<R: Read + Seek> AnvilChunkProvider for ZipChunkProvider<R> {
    fn load_chunk(&mut self, chunk_x: i32, chunk_z: i32) -> Result<CompoundTag, ChunkLoadError> {
        let RegionAndOffset {
            region_x,
            region_z,
            region_chunk_x,
            region_chunk_z,
        } = RegionAndOffset::from_chunk(chunk_x, chunk_z);

        let region_path = Self::region_path(region_x, region_z);

        let mut region_file = match self.zip_archive.by_name(&region_path) {
            Ok(x) => x,
            Err(_e) => return Err(ChunkLoadError::RegionNotFound { region_x, region_z }),
        };

        let uncompressed_size = region_file.size();
        let mut buf = Vec::with_capacity(uncompressed_size as usize);
        region_file.read_to_end(&mut buf)?;

        // TODO: Cache region files.
        // Warning: All the writes will be lost!
        // AnvilRegion needs Read+Seek+Write access to the reader
        // But ZipArchive only provides Read access to the compressed files
        // So we uncompress the file into memory, and pass the in-memory buffer
        // to AnvilRegion
        let mut region = AnvilRegion::new(Cursor::new(buf))?;

        region.read_chunk(region_chunk_x, region_chunk_z)
    }

    fn save_chunk(
        &mut self,
        _chunk_x: i32,
        _chunk_z: i32,
        _chunk_compound_tag: CompoundTag,
    ) -> Result<(), ChunkSaveError> {
        panic!("Writing to ZIP archives is not supported");
    }
}

