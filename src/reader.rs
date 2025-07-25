use std::io::Cursor;

use crate::{column::{Schema, Value}, ext::ReadExt, Error};

pub struct BsvReader<'a, R: ReadExt> {
    reader: &'a mut R,
    pub header: AnonymousSchemaBsvHeader,

    row_index: u64,
    row_buffer: Vec<Value>,
    broken: bool
}

impl<'a, R: ReadExt> BsvReader<'a, R> {
    pub fn new(reader: &'a mut R) -> Result<Self, Error> {
        let mut buffer = [0u8; 2];

        reader.read_exact(&mut buffer)?;
        let magic = buffer[0];
        let bsv_format = buffer[1];
        if magic != 0xBF {
            return Err(Error::InvalidMagic);
        }
        if bsv_format & 0xf != 1 { // AnonymousSchemaBSV
            return Err(Error::Unimplemented("apriori BSV"));
        }

        let header = AnonymousSchemaBsvHeader::from_reader(reader)?;
        let row_buffer = Vec::with_capacity(header.schemas.len());
        Ok(Self {
            reader,
            header,

            row_index: 0,
            row_buffer,
            broken: false
        })
    }

    pub fn next(&mut self) -> Result<Option<&Vec<Value>>, Error> {
        if self.row_index == self.header.row_count {
            return Ok(None);
        }

        if self.broken {
            return Err(Error::BrokenReader);
        }

        self.row_buffer.clear();
        for schema in self.header.schemas.iter() {
            self.row_buffer.push(
                schema.read(self.reader)
                    .inspect_err(|_| self.broken = true)?
            );
        }

        self.row_index += 1;
        Ok(Some(&self.row_buffer))
    }

    pub fn read_all(&mut self) -> Result<Vec<Vec<Value>>, Error> {
        let mut rows = Vec::with_capacity(self.header.row_count as _);
        while let Some(row) = self.next()? {
            rows.push(row.clone());
        }

        Ok(rows)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AnonymousSchemaBsvHeader {
    pub size: u16,
    pub row_count: u64,
    pub max_row_size: u64,
    pub schema_version: u32,
    pub schema_count: u32,
    pub schemas: Vec<Schema>
}

impl AnonymousSchemaBsvHeader {
    pub fn from_reader<R: ReadExt>(reader: &mut R) -> Result<Self, Error> {
        let mut size_buffer = [0u8; 2];
        reader.read_exact(&mut size_buffer)?;

        let size = u16::from_be_bytes(size_buffer);
        let mut buffer = vec![0u8; size as _];
        reader.read_exact(&mut buffer)?;

        let mut reader = Cursor::new(&buffer);
        let row_count = reader.read_vlq(8)?;
        let max_row_size = reader.read_vlq(8)?;
        let schema_version = reader.read_vlq(4)? as u32;
        let schema_count = reader.read_vlq(4)? as u32;
        let schemas = (0..schema_count)
            .map(|_| Schema::from_reader(&mut reader))
            .collect::<Result<Vec<Schema>, Error>>()?;

        Ok(Self {
            size,
            row_count,
            max_row_size,
            schema_version,
            schema_count,
            schemas
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::column::{ValueType, Value};

    use super::*;

    // this is not a real manifest file! contains dummy values
    const ROOT_MANIFEST_BSV: &[u8] = &[
        0xbf, 0x11, 0x00, 0x08, 0x03, 0x18, 0x20, 0x03, 0x40,
        0x12, 0x21, 0x08, 0x57, 0x69, 0x6e, 0x64, 0x6f, 0x77,
        0x73, 0x00, 0x84, 0x30, 0xa8, 0x3c, 0x12, 0xcc, 0x84,
        0x08, 0xd7, 0x1c, 0x69, 0x4f, 0x53, 0x00, 0x84, 0x31,
        0xab, 0xed, 0x7d, 0x96, 0x12, 0xce, 0xcd, 0x51, 0x41,
        0x6e, 0x64, 0x72, 0x6f, 0x69, 0x64, 0x00, 0x84, 0x32,
        0x40, 0xe3, 0x5b, 0x38, 0x80, 0x31, 0x31, 0x6f
    ];

    #[test]
    fn read_root_manifest() {
        let mut r = Cursor::new(ROOT_MANIFEST_BSV);
        let mut reader = BsvReader::new(&mut r).unwrap();
        assert_eq!(reader.header.schemas, vec![
            Schema { value_type: ValueType::Text, fixed_size: None },
            Schema { value_type: ValueType::ULong, fixed_size: None },
            Schema { value_type: ValueType::UNumFixed, fixed_size: Some(8) }
        ]);

        let rows = reader.read_all().unwrap();
        assert_eq!(rows, vec![
            [Value::Text("Windows".to_string()), Value::ULong(560), Value::UNumFixed(12122584966572332828)],
            [Value::Text("iOS".to_string()), Value::ULong(561), Value::UNumFixed(12388696233480211793)],
            [Value::Text("Android".to_string()), Value::ULong(562), Value::UNumFixed(4675681136367710575)]
        ]);

        // check if reader stopped correctly
        assert!(reader.next().is_ok_and(|opt| opt.is_none()));
    }
}