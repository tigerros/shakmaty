#![no_main]

use std::{
    io,
    path::{Path, PathBuf},
    sync::Arc,
};

use libfuzzer_sys::fuzz_target;
use shakmaty::{CastlingMode, Chess, fen::Fen};
use shakmaty_syzygy::{
    Tablebase,
    filesystem::{Filesystem, RandomAccessFile, ReadHint},
};

struct FakeFilesystem {
    data: Box<[u8]>,
}

impl Filesystem for FakeFilesystem {
    fn regular_file_size(&self, _path: &Path) -> io::Result<u64> {
        Ok(self.data.len() as u64)
    }

    fn read_dir(&self, _path: &Path) -> io::Result<Vec<PathBuf>> {
        Ok(vec!["KNvKP.rtbw".into()])
    }

    fn open(&self, _path: &Path) -> io::Result<Box<dyn RandomAccessFile>> {
        Ok(Box::new(FakeFile {
            data: self.data.clone(),
        }))
    }
}

struct FakeFile {
    data: Box<[u8]>,
}

impl RandomAccessFile for FakeFile {
    fn read_at(&self, buf: &mut [u8], offset: u64, _hint: ReadHint) -> io::Result<usize> {
        let offset = offset as usize;
        let end = offset + buf.len();
        buf.copy_from_slice(
            self.data
                .get(offset..end)
                .ok_or(io::ErrorKind::UnexpectedEof)?,
        );
        Ok(buf.len())
    }
}

fuzz_target!(|data: &[u8]| {
    let pos: Chess = "8/2K5/8/8/8/8/3p4/1k2N3 b - - 0 1" // KNvKP
        .parse::<Fen>()
        .expect("valid fen")
        .into_position(CastlingMode::Standard)
        .expect("valid position");

    let tables = Tablebase::with_filesystem(Arc::new(FakeFilesystem { data: data.into() }));
    let _ = tables.probe_wdl(&pos);
});
