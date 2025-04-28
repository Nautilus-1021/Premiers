use std::{fs::File, io::{self, BufRead, BufWriter, ErrorKind, Read, Write}, ops::Add};

use flate2::{bufread::GzDecoder, write::GzEncoder, Compression};

pub struct PrimesExport(GzEncoder<BufWriter<File>>, [u8; 4]);

impl PrimesExport {
    pub fn new(filename: &str) -> io::Result<Self> {
        let fl = File::create(filename)?;

        Ok(Self(GzEncoder::new(BufWriter::new(fl), Compression::best()), [0; 4]))
    }

    pub fn export_prime(&mut self, prime: u32) -> io::Result<()> {
        self.0.write(u32_varint(prime, &mut self.1))?;
        Ok(())
    }

    pub fn close(mut self) -> io::Result<()> {
        self.0.flush()?;

        Ok(())
    }
}

pub struct PrimesIter<R: BufRead>(GzDecoder<R>);

impl<R: BufRead> PrimesIter<R> {
    pub fn new(reader: R) -> Self {
        Self(GzDecoder::new(reader))
    }
}

impl<R: BufRead> Iterator for PrimesIter<R> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        match load_varint(&mut self.0) {
            Ok(nmb) => Some(nmb),
            Err(errmsg) => {
                if errmsg.kind() == ErrorKind::UnexpectedEof {
                    return None
                } else {
                    panic!("An error occured while trying to read the primes file: {}", errmsg)
                }
            }
        }
    }
}

fn u32_varint(mut nmb: u32, pbuf: &mut [u8; 4]) -> &[u8] {
    let bits_needed = f64::from(nmb).log2().add(1.0).floor() as usize;

    for idx in 0..((bits_needed - 1) / 7 + 1) {
        pbuf[idx] = (nmb & 0b1111111) as u8 | 0b10000000;
        nmb >>= 7;
    }

    pbuf[(bits_needed - 1) / 7] &= 0b1111111;

    &pbuf[..((bits_needed - 1) / 7 + 1)]
}

fn load_varint(file: &mut impl Read) -> io::Result<u32> {
    let mut nmb = 0;

    let mut buf = [0];
    for x in 0..4 {
        file.read_exact(&mut buf)?;

        nmb |= (buf[0] as u32 & 0b01111111) << (x * 7);
        if (buf[0] & 0b10000000) == 0 {
            break;
        }
    }

    Ok(nmb)
}

#[cfg(test)]
mod tests {
    use super::u32_varint;

    const ESL: [u8; 4] = [0; 4];

    #[test]
    fn test1() {
        const RES: [u8; 1] = [0b00000001];
        let mut buf = ESL;
        assert_eq!(u32_varint(1, &mut buf), &RES);
    }

    #[test]
    fn test2() {
        const RES: [u8; 2] = [0b10000000, 0b00000010];
        let mut buf = ESL;
        assert_eq!(u32_varint(256, &mut buf), &RES);
    }

    #[test]
    fn test3() {
        const RES: [u8; 1] = [0b01000011];
        let mut buf = ESL;
        assert_eq!(u32_varint(67, &mut buf), &RES);
    }
}