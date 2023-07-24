
use std::error::Error;
use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use crate::Chunk;

//Stream going to be used to assign to every png file to sequentially read data
#[derive(Debug)]
struct Stream {
    sequential_counter: usize,
}

impl Stream {
    fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
    //Reads bytes sequentially and updates a counter every time we read bytes
    fn read_bytes_sequential(&mut self, byte_list: &Vec<u8>, range: usize) -> Result<Vec<u8>, Box<dyn Error>> {
        let start = self.sequential_counter;
        let end = self.sequential_counter + range;
        if byte_list.len() >= end {
            self.sequential_counter += range;
            Ok(byte_list[start..end].to_vec())
        } else {
            Err("Range is out of bounds".into())
        }
    }
}

impl Default for Stream {
    fn default() -> Stream {
        Stream {
            sequential_counter: 0,
        }
    }
}

#[derive(Debug)]
pub struct Png<'a> {
    file: FileLoader<'a>,
    data_stream: Stream,
    pub chunk_list: Vec<Chunk>,
    signature_verified: bool,
    png_signature: Vec<u8>,
}

impl<'a> Png<'a> {
    pub fn new(file_name: &'a str) -> Self {
        let file = FileLoader::load_file(&file_name).expect("Failed to open file");
        let mut stream = Stream::new();
        let signature = &stream
            .read_bytes_sequential(&file.data, 8)
            .expect("Failed to read bytes");
        let mut verified = false;
        let mut chunk_list = Vec::new();
        if signature == &vec![137, 80, 78, 71, 13, 10, 26, 10] {
            verified = true;
        }
        Self {
            file: file,
            data_stream: stream,
            chunk_list: chunk_list,
            signature_verified: verified,
            png_signature: [137, 80, 78, 71, 13, 10, 26, 10].to_vec(),
        }
    }

    pub fn get_string(&mut self, length: usize) -> Result<String, Box<dyn Error>> {
        let bytes = self.read_bytes(length)?;
        String::from_utf8(bytes).map_err(Into::into)
    }


    pub fn add_chunk(&mut self, chunk: Chunk) -> Result<(), Box<dyn Error>> {
        self.chunk_list.push(chunk);
        Ok(())
    }
    pub fn read_bytes(&mut self, range: usize) -> Result<Vec<u8>, Box<dyn Error>> {
        self.data_stream.read_bytes_sequential(&self.file.data, range)
    }

    pub fn big_endian_u32(&mut self) -> Result<u32, Box<dyn Error>> {
        let bytes = self.read_bytes(4)?;
        if bytes.len() != 4 {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Not enough bytes to read a u32",)));
        }

        Ok(((bytes[0] as u32) << 24)
            | ((bytes[1] as u32) << 16)
            | ((bytes[2] as u32) << 8)
            | (bytes[3] as u32))
    }

    pub fn big_endian_u16(&mut self) -> Result<u16, Box<dyn Error>> {
        let bytes = self.read_bytes(2)?;
        if bytes.len() != 2 {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Not enough bytes to read a u16",)));
        }

        Ok(((bytes[0] as u16) << 8)
            | (bytes[1] as u16))
    }

    pub fn get_u32(&mut self) -> Result<Vec<u8>, Box<dyn Error>> {
        let bytes = self.read_bytes(4)?;
        if bytes.is_empty() {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Not enough bytes to read a u32",)));
        }
        Ok(bytes)
    }

    pub fn get_u16(&mut self) -> Result<Vec<u8>, Box<dyn Error>> {
        let bytes = self.read_bytes(2)?;
        if bytes.is_empty() {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Not enough bytes to read a u16")));
        }
        Ok(bytes)
    }

    pub fn get_u8(&mut self) -> Result<u8, Box<dyn Error>> {
        let bytes = self.read_bytes(1)?;
        if bytes.is_empty() {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Not enough bytes to read a u8")));
        }
        Ok(bytes[0])
    }

    pub fn read_null_terminated_string(&mut self) -> Result<(String, u32), Box<dyn Error>> {
        let mut bytes = Vec::new();
        let mut byte = self.get_u8()?;
        while byte != 0 {
            bytes.push(byte);
            byte = self.get_u8()?;
        }
        let length = bytes.len() as u32;
        let string = String::from_utf8(bytes)?;
        Ok((string, length))
    }


    fn verify_signature(mut self) -> Self {
        let mut buf = vec![0; 8]; //8 Byte buff
        let mut file = File::open(&self.file.file_name).expect("Can't open file");
        file.read_exact(&mut buf).expect("Can't read from file");

        if buf == vec![137, 80, 78, 71, 13, 10, 26, 10] {
            println!("Signature is correct");
            self.signature_verified = true;
        }

        self
    }
}

//idk why I've decided to use lifetimes here but I wanted to use the str variable so I'm forced to, only using this shit because it's stack allocated instead of heap
//Seperate struct so in the future I can handle file loads and deloads for potential optimisation/error checking
#[derive(Debug)]
struct FileLoader<'a> {
    file_name: &'a str,
    data: Vec<u8>,
}

impl<'a> FileLoader<'a> {
    fn load_file(f_name: &'a str) -> Result<Self, std::io::Error> {
        let mut file_data = File::open(f_name)?;
        let mut buffer = Vec::new();
        file_data.read_to_end(&mut buffer)?;
        Ok(Self {
            file_name: f_name,
            data: buffer,
        })
    }

    fn get_extension_from_filename(&self) -> Option<&str> {
        Path::new(self.file_name)
            .extension()
            .and_then(OsStr::to_str)
    }
}
