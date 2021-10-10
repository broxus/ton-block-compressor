use std::io;

use anyhow::Result;

pub struct ZstdWrapper {
    d_dict: zstd::dict::DecoderDictionary<'static>,
    c_dict: zstd::dict::EncoderDictionary<'static>,
    buffer: Vec<u8>,
}

impl ZstdWrapper {
    pub fn new() -> Self {
        Self::with_level(zstd::DEFAULT_COMPRESSION_LEVEL)
    }

    pub fn with_level(level: i32) -> Self {
        let d_dict = zstd::dict::DecoderDictionary::copy(include_bytes!("../dictionary"));
        let c_dict = zstd::dict::EncoderDictionary::copy(include_bytes!("../dictionary"), level);
        Self {
            c_dict,
            d_dict,
            buffer: Vec::with_capacity(1024 * 1024),
        }
    }

    pub fn compress(&mut self, bytes: &[u8]) -> Result<&[u8]> {
        let mut wrapper = io::Cursor::new(bytes);
        let mut output_wrapper = io::Cursor::new(&mut self.buffer);

        let mut encoder =
            zstd::stream::Encoder::with_prepared_dictionary(&mut output_wrapper, &self.c_dict)?;
        io::copy(&mut wrapper, &mut encoder)?;
        encoder.finish()?;
        let out_pos = output_wrapper.position() as usize;
        Ok(&self.buffer[0..out_pos])
    }
    // pub fn compress(&mut self, bytes: &[u8]) -> Result<Vec<u8>> {
    //     let mut wrapper = io::Cursor::new(bytes);
    //
    //     let mut buffer = Vec::with_capacity(1024 * 1024);
    //     let mut output_wrapper = io::Cursor::new(&mut buffer);
    //     // let mut output_wrapper = io::Cursor::new(&mut self.buffer);
    //     let mut encoder =
    //         zstd::stream::Encoder::with_prepared_dictionary(&mut output_wrapper, &self.c_dict)?;
    //     io::copy(&mut wrapper, &mut encoder)?;
    //     encoder.finish()?;
    //     // Ok(&self.buffer[0..out_pos])
    //     Ok(buffer)
    // }

    pub fn decompress(&mut self, bytes: &[u8]) -> Result<&[u8]> {
        self.buffer.truncate(0);
        let mut wrapper = io::Cursor::new(bytes);
        let mut decoder =
            zstd::stream::Decoder::with_prepared_dictionary(&mut wrapper, &self.d_dict)?;
        let mut output_wrapper = io::Cursor::new(&mut self.buffer);
        io::copy(&mut decoder, &mut output_wrapper)?;
        let out_pos = output_wrapper.position() as usize;
        drop(output_wrapper);
        Ok(&self.buffer[0..out_pos])
    }
}

impl Default for ZstdWrapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use std::io::Read;

    use rand::Rng;
    use zstd::stream::read::Decoder;

    use crate::ZstdWrapper;

    #[test]
    fn test_encode() {
        let mut encoder = ZstdWrapper::new();
        let input = b"asasaasasasasasasasasasaaaaaaaaaaaaasassas";
        let res = encoder.compress(&input[..]).unwrap();
        let mut de = Decoder::with_dictionary(&res[..], include_bytes!("../dictionary")).unwrap();
        let mut out = vec![];
        de.read_to_end(&mut out).unwrap();
        assert_eq!(&out, input);
    }

    #[test]
    fn test_encode_rand() {
        let mut encoder = ZstdWrapper::new();
        let mut expected = vec![0; 1024 * 10];
        rand::thread_rng().fill(expected.as_mut_slice());

        let res = encoder.compress(&expected).unwrap().to_vec();
        println!("Len: {}", res.len());

        let got = encoder.decompress(&res).unwrap();

        assert_eq!(expected, got);
    }
}
