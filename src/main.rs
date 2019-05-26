extern crate clap;
extern crate rand;
#[macro_use] extern crate failure;

use std::io::prelude::*;
use std::io::{BufReader, Cursor};
use std::fs::File;
use std::fmt;


#[derive(Debug, Fail)]
enum JPGError {
    #[fail(display = "SOI is not found error")]
    SOINotFoundError,
    #[fail(display = "APP0 is not found error")]
    APP0NotFoundError,
    #[fail(display = "DQT is not found error")]
    DQTNotFoundError,
    #[fail(display = "SOF0 is not found error")]
    SOF0NotFoundError,
    #[fail(display = "DHT is not found error")]
    DHTNotFoundError,
    #[fail(display = "SOS is not found error")]
    SOSNotFoundError,
    #[fail(display = "ImageBody is not found error")]
    ImageBodyNotFound,
    #[fail(display = "Broken Jpeg file")]
    BrokenJpgFileError
}

use clap::{App, Arg};

#[derive(Clone)]
struct JPG {
    pub soi: [u8;2],
    pub app0: [u8;18],
    pub dqt: [u8;69 * 2],
    pub sof0: [u8;19],
    pub dht: [u8;33 + 183 + 33 + 183],
    pub sos: [u8;14],
    pub image: std::vec::Vec<u8>,
}

impl fmt::Display for JPG {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        fn formatter(b: &u8) -> String {
            format!("{:x} ", b)
        }
        let soi = &self.soi.iter().map(formatter).collect::<String>();
        let app0 = &self.app0.iter().map(formatter).collect::<String>();
        let dqt = &self.dqt.iter().map(formatter).collect::<String>();
        let sof0 = &self.sof0.iter().map(formatter).collect::<String>();
        let dht = &self.dht.iter().map(formatter).collect::<String>();
        let sos = &self.sos.iter().map(formatter).collect::<String>();
        let image = &self.image.iter().map(formatter).collect::<String>();
        write!(
            f,
            "
soi: {}
app0: {}
dqt: {}
sof0: {}
dht: {}
sos: {}
image: {}
            ",
            soi,
            app0,
            dqt,
            sof0,
            dht,
            sos,
            image
        )
    }
}


fn parse(binary: &std::vec::Vec<u8>) -> Result<JPG, JPGError> {
    let mut cur = Cursor::new(binary);

    let mut soi_buffer = [false as u8;2];
    let mut app0_buffer = [false as u8;18];
    let mut dqt_buffer = [false as u8 ;69 * 2];
    let mut sof0_buffer = [false as u8; 19];
    let mut dht_buffer = [false as u8; 33 + 183 + 33 + 183];
    let mut sos_buffer = [false as u8;14];
    let mut other_buffer = Vec::new();

    match cur.read_exact(&mut soi_buffer) {
        Ok(_) => {}
        Err(_) => return Err(JPGError::SOINotFoundError)
    };
    match cur.read_exact(&mut app0_buffer) {
        Ok(_) => {}
        Err(_) => return Err(JPGError::APP0NotFoundError)
    };
    match cur.read_exact(&mut dqt_buffer) {
        Ok(_) => {}
        Err(_) => return Err(JPGError::DQTNotFoundError)
    }
    match cur.read_exact(&mut sof0_buffer) {
        Ok(_) => {}
        Err(_) => return Err(JPGError::SOF0NotFoundError)
    };
    match cur.read_exact(&mut dht_buffer) {
        Ok(_) => {}
        Err(_) => return Err(JPGError::DHTNotFoundError)
    };
    match cur.read_exact(&mut sos_buffer) {
        Ok(_) => {}
        Err(_) => return Err(JPGError::SOSNotFoundError)
    };
    match cur.read_to_end(&mut other_buffer) {
        Ok(_) => {}
        Err(_) => return Err(JPGError::ImageBodyNotFound)
    };
    let buffer_iter = &mut other_buffer.into_iter();
    let mut image_body: Vec<u8> = Vec::new();

    while let (Some(byte), next_byte) = (buffer_iter.next(), buffer_iter.next()) {
        match (byte, next_byte) {
            (b, Some(nb)) => {
                if b == 255 as u8 && nb == 217 as u8 {
                    break;
                } else {
                    image_body.push(b);
                    image_body.push(nb);
                }
            }
            (_, None) => {
                return Err(JPGError::BrokenJpgFileError)
            }
        }
    }

    return Ok(JPG {
        soi: soi_buffer,
        app0: app0_buffer,
        dqt: dqt_buffer,
        sof0: sof0_buffer,
        dht: dht_buffer,
        sos: sos_buffer,
        image: image_body
    });
}

fn f(b: &u8) -> u8 {
    if *b == 232 as u8 {
        return *b - 1 as u8;
    }
    *b
}
fn break_jpg(_rng: &mut rand::prelude::ThreadRng, jpg: JPG) -> JPG{
    let broken_bytes = jpg.image.iter().map(f).collect::<Vec<u8>>();
    JPG {
        soi: jpg.soi,
        app0: jpg.app0,
        dqt: jpg.dqt,
        sof0: jpg.sof0,
        dht: jpg.dht,
        sos: jpg.sos,
        image:broken_bytes
    }
}

fn create_binary(jpg: JPG) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::new();
    bytes.extend_from_slice(jpg.soi.iter().as_slice());
    bytes.extend_from_slice(jpg.app0.iter().as_slice());
    bytes.extend_from_slice(jpg.dqt.iter().as_slice());
    bytes.extend_from_slice(jpg.sof0.iter().as_slice());
    bytes.extend_from_slice(jpg.dht.iter().as_slice());
    bytes.extend_from_slice(jpg.sos.iter().as_slice());
    bytes.extend_from_slice(jpg.image.iter().as_slice());
    bytes.extend_from_slice(&[255 as u8, 217 as u8]);
    return bytes
}

fn main() -> std::io::Result<()> {
    let mut rng = rand::thread_rng();
    let matches = App::new("jpg-glitch")
        .version("1.0")
        .author("himanoa <matsunoappy@gmail.com>")
        .arg(
            Arg::with_name("INPUT")
            .help("Sets the input file to use")
            .required(true)
            .index(1),
            )
        .get_matches();
    let f = File::open(matches.value_of("INPUT").unwrap())?;
    let mut buffer = vec![];
    let mut reader = BufReader::new(f);
    reader.read_to_end(&mut buffer)?;
    let a = parse(&buffer).unwrap();
    let mut out = std::io::stdout();
    out.write_all(create_binary(break_jpg(&mut rng, a)).as_slice())?;
    out.flush()?;

    Ok(())

}
