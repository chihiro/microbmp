// µbmp - Tiny library for reading bitmap pixel data.

use std::io::prelude::*;
use std::fs::File;
use std::io;
use std::intrinsics::transmute;

// Pixel enumerated type containing each BPP.
#[derive(Debug, Clone)]
pub enum Pixel {
  ABGR(u8, u8, u8, u8),
  BGR(u8, u8, u8),
  PaletteColor(u8)
}

// Enum for each compression method.
#[derive(Debug, Clone)]
pub enum CompressionMethod {
  None,
  Rle8Bit,
  Rle4Bit,
  Huffman1D,
  Jpeg,
  Png,
  Other(u32)
}

// A basic (and incomplete) BITMAPV5HEADER.
#[derive(Debug, Clone)]
pub struct BitmapV5Header {
  pub size: u32,
  pub pix_width: i32,
  pub pix_height: i32,
  pub bpp: u16,
  pub method: CompressionMethod,
  pub colors: u32
}

// Containing Bitmap structure.
#[derive(Debug, Clone)]
pub struct Bitmap {
  pub data: Vec<u8>,
  pub size: u32,
  pub offset: u32,
  pub header: BitmapV5Header,
  pub pixels: Vec<Pixel>
}

pub type BitmapResult<T> = Result<T, BitmapError>;

#[derive(Debug)]
pub enum BitmapError {
  InvalidBitmapData,
  UnsupportedBitsPerPixel,
  BitmapIOError(io::Error)
}

impl std::convert::From<io::Error> for BitmapError {
  fn from(err: io::Error) -> BitmapError {
    BitmapError::BitmapIOError(err)
  }
}

impl Bitmap {
  pub fn new(file: &mut File) -> BitmapResult<Bitmap> {
    let mut buf: Vec<u8> = Vec::new();    
    try!(file.read_to_end(&mut buf));

    // magic number check
    if &buf[0..2] != b"BM" {
      return Err(BitmapError::InvalidBitmapData)
    }

    let size: u32 = unsafe {
      let mut bytes: [u8; 4] = [0; 4];
      bytes.clone_from_slice(&buf[2..6]);
      transmute(bytes)
    };

    let offset: u32 = unsafe {
      let mut bytes: [u8; 4] = [0; 4];
      bytes.clone_from_slice(&buf[10..14]);
      transmute(bytes)
    };

    let header_size: u32 = unsafe {
      let mut bytes: [u8; 4] = [0; 4];
      bytes.clone_from_slice(&buf[14..18]);
      transmute(bytes)
    };

    let pix_width: i32 = unsafe {
      let mut bytes: [u8; 4] = [0; 4];
      bytes.clone_from_slice(&buf[18..22]);
      transmute(bytes)
    };

    let pix_height: i32 = unsafe {
      let mut bytes: [u8; 4] = [0; 4];
      bytes.clone_from_slice(&buf[22..26]);
      transmute(bytes)
    };
    
    let bpp: u16 = unsafe {
      let mut bytes: [u8; 2] = [0; 2];
      bytes.clone_from_slice(&buf[28..30]);
      transmute(bytes)
    };

    let method: CompressionMethod = unsafe {
      let mut bytes: [u8; 4] = [0; 4];
      bytes.clone_from_slice(&buf[30..34]);
      match transmute(bytes) {
        0 => CompressionMethod::None,
        1 => CompressionMethod::Rle8Bit,
        2 => CompressionMethod::Rle4Bit,
        3 => CompressionMethod::Huffman1D,
        4 => CompressionMethod::Jpeg,
        5 => CompressionMethod::Png,
        n => CompressionMethod::Other(n)
      }
    }; 
    
    let end: u32 = offset + unsafe {
      let mut bytes: [u8; 4] = [0; 4];
      bytes.clone_from_slice(&buf[34..38]);
      transmute::<[u8; 4], u32>(bytes)
    };

    let colors: u32 = unsafe {
      let mut bytes: [u8; 4] = [0; 4];
      bytes.clone_from_slice(&buf[46..50]);
      transmute(bytes)
    };

    let pixel_data = match bpp {
      24 | 32 => {
        buf[offset as usize .. end as usize]
          .chunks(4)
          .map(|slice| {
            Pixel::ABGR(slice[0], slice[1], slice[2], slice[3])
          })
          .collect::<Vec<_>>()
      }

      4 => {
        fn nibbles(n: u8) -> Vec<u8> {
          vec![(n & 0xF0) >> 4, n & 0xF]
        }

        buf[offset as usize .. end as usize]
          .iter()
          .flat_map(|pixel| nibbles(*pixel))
          .map(|pixel| Pixel::PaletteColor(pixel))
          .collect::<Vec<_>>()
      }

      _ => { return Err(BitmapError::UnsupportedBitsPerPixel) }
    };
    
    Ok(Bitmap {
      data: buf,
      size: size,
      offset: offset,
      header: BitmapV5Header {
        size: header_size,
        pix_width: pix_width,
        pix_height: pix_height,
        bpp: bpp,
        method: method,
        colors: colors
      },
      pixels: pixel_data
    })
  }
}
