use std::io::{Error, ErrorKind, Read};

pub trait ReadExt: Read {
    fn read_vlq(&mut self, max_bytes: usize) -> std::io::Result<u64> {
        let mut buffer = [0u8; 1];
        let mut value = 0u64;
        let mut i = 0;

        loop {
            self.read_exact(&mut buffer)?;
            let byte = buffer[0];
            value = (value << 7) | (0x7f & byte) as u64;
            if byte >> 7 == 0 {
                break;
            }

            i += 1;
            if max_bytes == i {
                // max bytes reached but VLQ is still continuing
                return Err(Error::new(ErrorKind::InvalidData, "VLQ exceeds max size"));
            }
        }

        Ok(value)
    }

    fn read_u8(&mut self) -> std::io::Result<u8> {
        let mut buffer = [0u8; 1];
        self.read_exact(&mut buffer)?;
        Ok(buffer[0])
    }

    fn read_u16_be(&mut self) -> std::io::Result<u16> {
        let mut buffer = [0u8; 2];
        self.read_exact(&mut buffer)?;
        Ok(u16::from_be_bytes(buffer))
    }

    fn read_unum(&mut self, len: usize) -> std::io::Result<u64> {
        let mut buffer = vec![0u8; len];
        self.read_exact(&mut buffer)?;

        let mut value = 0;
        for x in buffer {
            value = (value << 8) | x as u64;
        }
        Ok(value)
    }

    fn read_blob(&mut self, len: usize) -> std::io::Result<Vec<u8>> {
        let mut buffer = vec![0u8; len];
        self.read_exact(&mut buffer)?;
        Ok(buffer)
    }

    fn read_null_terminated_string(&mut self) -> std::io::Result<String> {
        let mut buffer = Vec::with_capacity(16);
        let mut byte = [0u8; 1];
        loop {
            self.read_exact(&mut byte)?;
            if byte[0] != 0 {
                buffer.push(byte[0]);
            }
            else {
                break;
            }
        }

        String::from_utf8(buffer)
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))
    }

    fn skip(&mut self, len: usize) -> std::io::Result<()> {
        self.read_exact(&mut Vec::with_capacity(len))
    }
}

impl<R: Read + ?Sized> ReadExt for R {}