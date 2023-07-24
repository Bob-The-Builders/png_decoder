#![allow(dead_code)]
#![allow(unused_variables)]

use std::collections::HashMap;
use std::error::Error;
mod png;
use crate::png::png::Png;
use crate::png::{IDHRChunk, PLTEChunk, IDATChunk, IENDChunk, tIMEChunk, bKGDChunk, gAMAChunk, cHRMChunk, dSIGChunk, eXIfChunk, hISTChunk,
    iCCPChunk, iTXtChunk, pHYsChunk, sBITChunk, sPLTChunk, sRGBChunk, sTERChunk, tEXtChunk, tRNSChunk, zTXtChunk, Chunk};
use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use std::path::Path;


//TODO: Implement decompression, defiltering and interlacing methods
fn main() {
    let mut png = Png::new(""); //Add path here
    //println!("{:?}", png);
    let mut png_decoder = PngDecoder::new(png);
    png_decoder.get_all_chunks();
    println!("{:?}", png_decoder.png_file.chunk_list)
}

#[derive(Debug)]
struct PngDecoder<'a> {
    png_file: Png<'a>,
    chunk_type_map: HashMap<Vec<u8>, String>,
}

//Will eventually make this so a list of png files will be used for mass editing all over one decoder

/*Every Chunk's type is represented by an characters in ascii code here we will use an hashtable to get what type of chunk it is based on ascii code
therefore we have two options here, convert out bytes list into ascii and just return the string, or match the ascii code with a set list of strings in ascii
I'm going to match the ascii with a set list of ascii codes as we won't have to worry about modified files at all then*/
impl<'a> PngDecoder<'a> {
    fn new(png_file: Png<'a>) -> Self {
        let mut chunk_type_map = HashMap::new();
        chunk_type_map.insert(vec![73, 72, 68, 82], "IDHR".to_string());
        chunk_type_map.insert(vec![73, 68, 65, 84], "IDAT".to_string());
        chunk_type_map.insert(vec![80, 76, 84, 69], "PLTE".to_string());
        chunk_type_map.insert(vec![98, 75, 71, 68], "bKGD".to_string());
        chunk_type_map.insert(vec![99, 72, 82, 77], "cHRM".to_string());
        chunk_type_map.insert(vec![100, 83, 73, 71], "dSIG".to_string());
        chunk_type_map.insert(vec![101, 88, 73, 102], "eXIf".to_string());
        chunk_type_map.insert(vec![103, 65, 77, 65], "gAMA".to_string());
        chunk_type_map.insert(vec![104, 73, 83, 84], "hIST".to_string());
        chunk_type_map.insert(vec![105, 67, 67, 80], "iCCP".to_string());
        chunk_type_map.insert(vec![105, 84, 88, 116], "iTXt".to_string());
        chunk_type_map.insert(vec![112, 72, 89, 115], "pHYs".to_string());
        chunk_type_map.insert(vec![115, 66, 73, 84], "sBIT".to_string());
        chunk_type_map.insert(vec![115, 80, 76, 84], "sPLT".to_string());
        chunk_type_map.insert(vec![115, 82, 71, 66], "sRGB".to_string());
        chunk_type_map.insert(vec![115, 84, 69, 82], "sTER".to_string());
        chunk_type_map.insert(vec![116, 69, 88, 116], "tEXt".to_string());
        chunk_type_map.insert(vec![116, 73, 77, 69], "tIME".to_string());
        chunk_type_map.insert(vec![116, 82, 78, 83], "tRNS".to_string());
        chunk_type_map.insert(vec![122, 84, 88, 116], "zTXt".to_string());
        chunk_type_map.insert(vec![73, 69, 78, 68], "IEND".to_string());

        Self {png_file, chunk_type_map}
    }

    fn get_all_chunks(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            let length = self.png_file.big_endian_u32()?;

            let key_bytes = self.png_file.read_bytes(4)?;
            println!("{:?}", key_bytes);
            let chunk_type = self.chunk_type_map.get(&key_bytes)
                .ok_or_else(|| Box::<dyn Error>::from("Unexpected chunk type: None"))?;

            let chunk = match chunk_type.as_str() {
                "IDHR" => Chunk::IDHR(IDHRChunk::new(length, &mut self.png_file)?),
                "PLTE" => Chunk::PLTE(PLTEChunk::new(length, &mut self.png_file)?),
                "IDAT" => Chunk::IDAT(IDATChunk::new(length, &mut self.png_file)?),
                "tIME" => Chunk::tIME(tIMEChunk::new(length, &mut self.png_file)?),
                "gAMA" => Chunk::gAMA(gAMAChunk::new(length, &mut self.png_file)?),
                "cHRM" => Chunk::cHRM(cHRMChunk::new(length, &mut self.png_file)?),
                "bKGD" => Chunk::bKGD(bKGDChunk::new(length, &mut self.png_file)?),
                "tEXt" => Chunk::tEXt(tEXtChunk::new(length, &mut self.png_file)?),
                "dSIG" => Chunk::dSIG(dSIGChunk::new(length, &mut self.png_file)?),
                "eXIf" => Chunk::eXIf(eXIfChunk::new(length, &mut self.png_file)?),
                "hIST" => Chunk::hIST(hISTChunk::new(length, &mut self.png_file)?),
                "iCCP" => Chunk::iCCP(iCCPChunk::new(length, &mut self.png_file)?),
                "iTXt" => Chunk::iTXt(iTXtChunk::new(length, &mut self.png_file)?),
                "pHYs" => Chunk::pHYs(pHYsChunk::new(length, &mut self.png_file)?),
                "sBIT" => Chunk::sBIT(sBITChunk::new(length, &mut self.png_file)?),
                "sPLT" => Chunk::sPLT(sPLTChunk::new(length, &mut self.png_file)?),
                "sRGB" => Chunk::sRGB(sRGBChunk::new(length, &mut self.png_file)?),
                "sTER" => Chunk::sTER(sTERChunk::new(length, &mut self.png_file)?),
                "tRNS" => Chunk::tRNS(tRNSChunk::new(length, &mut self.png_file)?),
                "zTXt" => Chunk::zTXt(zTXtChunk::new(length, &mut self.png_file)?),
                "IEND" => {
                    let iend_chunk = Chunk::IEND(IENDChunk::new(length, &mut self.png_file)?);
                    self.png_file.add_chunk(iend_chunk)?;
                    break;
                }
                _ => return Err(Box::<dyn Error>::from(format!("Unexpected chunk type: {}", chunk_type))),
            };
            self.png_file.add_chunk(chunk)?;
        }
        Ok(())
    }

    fn sum_big_endian(bytes: &[u8]) -> Result<u32, Box<dyn Error>> {
        if bytes.len() != 4 {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Not enough bytes to read a u32",
            )));
        }

        Ok(((bytes[0] as u32) << 24)
            | ((bytes[1] as u32) << 16)
            | ((bytes[2] as u32) << 8)
            | (bytes[3] as u32))
    }
}



