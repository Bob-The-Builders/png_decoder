use std::error::Error;
use crate::Png;

/*IDHR must be the first chunk in the image and it contains:
- width (4 bytes)
- height (4 bytes)
- bit depth (1 byte, values 1, 2, 4, 8, or 16)
- color type (1 byte, values 0, 2, 3, 4, or 6)
- compression method (1 byte, value 0) Only ever one value
- filter method (1 byte, value 0) Only ever one value
- interlace method (1 byte, values 0 "no interlace" or 1 "Adam7 interlace") (13 data bytes total) - Wikipedia */
/*TODO add CRC to chunks:
3.4. CRC algorithm
Chunk CRCs are calculated using standard CRC methods with pre and post conditioning, as defined by ISO 3309 [ISO-3309] or ITU-T V.42 [ITU-V42]. The CRC polynomial employed is
   x^32+x^26+x^23+x^22+x^16+x^12+x^11+x^10+x^8+x^7+x^5+x^4+x^2+x+1
The 32-bit CRC register is initialized to all 1's, and then the data from each byte is processed from the least significant bit (1) to the most significant bit (128). After all the data bytes are processed, the CRC register is inverted (its ones complement is taken). This value is transmitted (stored in the file) MSB first. For the purpose of separating into bytes and ordering, the least significant bit of the 32-bit CRC is defined to be the coefficient of the x^31 term.
Practical calculation of the CRC always employs a precalculated table to greatly accelerate the computation. See Sample CRC Code.*/

#[derive(Debug)]
pub struct IDHRChunk {
    length: u32,
    width: u32,
    height: u32,
    bit_depth: u8,
    color_type: ColorType,
    compression_method: u8,
    filter_method: u8,
    interlace_method: InterlaceMethod,
    CRC: Vec<u8>,
}

impl IDHRChunk {
    pub fn new(length: u32, png_file: &mut Png) -> Result<Self, Box<dyn Error>> {
        let width = png_file.big_endian_u32()?;
        let height = png_file.big_endian_u32()?;
        let bit_depth = png_file.get_u8()?;
        let color_type = match png_file.get_u8()? {
            0 => ColorType::Grayscale,
            2 => ColorType::RGB,
            3 => ColorType::Indexed,
            4 => ColorType::GrayscaleAlpha,
            6 => ColorType::RGBA,
            _ => return Err("Unknown color type".into()),
        };
        let compression_method = png_file.get_u8()?;
        let filter_method = png_file.get_u8()?;
        let interlace_method = match png_file.get_u8()? {
            0 => InterlaceMethod::None,
            1 => InterlaceMethod::Adam7,
            _ => return Err("Unknown interlace method".into()),
        };
        let CRC = png_file.get_u32()?;

        Ok(Self{length, width, height, bit_depth, color_type, compression_method, filter_method, interlace_method, CRC})
    }
}

#[derive(Debug)]
pub enum ColorType {
    Grayscale,
    RGB,
    Indexed,
    GrayscaleAlpha,
    RGBA,
}

#[derive(Debug)]
enum InterlaceMethod {
    None,
    Adam7,
}



//PLTE Chunks a formed by a series of PaletteEntries
#[derive(Debug)]
pub struct PaletteEntry {
    red: u8,
    green: u8,
    blue: u8,
}

#[derive(Debug)]
pub struct PLTEChunk {
    length: u32,
    entries: Vec<PaletteEntry>,
    CRC: Vec<u8>,
}

impl PLTEChunk {
    pub fn new(length: u32, png_file: &mut Png) -> Result<Self, Box<dyn Error>> {
        if length % 3 != 0 {
            return Err("Invalid chunk length for PLTE".into());
        }

        let mut entries = Vec::new();

        for _ in 0..(length / 3) {
            let red = png_file.get_u8()?;
            let green = png_file.get_u8()?;
            let blue = png_file.get_u8()?;

            entries.push(PaletteEntry { red, green, blue });
        }
        
        let CRC = png_file.get_u32()?;

        Ok(Self{length, entries, CRC})
    }
}


//IDAT chunk (Contains all image data compressed and filtered)
#[derive(Debug)]
pub struct IDATChunk {
    length: u32,
    data: Vec<u8>,
    CRC: Vec<u8>,
}

impl IDATChunk {
    pub fn new(length: u32, png_file: &mut Png) -> Result<Self, Box<dyn Error>> {
        let mut data = Vec::new();

        for _ in 0..length {
            data.push(png_file.get_u8()?);
        }

        let CRC = png_file.get_u32()?;

        Ok(Self{length, data, CRC})
    }
}


//IEND
#[derive(Debug)]
pub struct IENDChunk {
    length: u32,
    CRC: Vec<u8>,
}

impl IENDChunk {
    pub fn new(length: u32, png_file: &mut Png) -> Result<Self, Box<dyn Error>> {
        let CRC = png_file.get_u32()?;
        Ok(Self {length, CRC })
    }
}

// bKGD
#[derive(Debug)]
pub struct bKGDChunk {
    length: u32,
    color: Color,
    CRC: Vec<u8>,
}

impl bKGDChunk {
    pub fn new(length: u32, png_file: &mut Png) -> Result<Self, Box<dyn Error>> {
        let IDHR_chunk_opt = png_file.chunk_list.iter_mut().find_map(|p| match p {
            Chunk::IDHR(chunk_list, ..) => Some(chunk_list),
            _ => None,
        });
        
        let IDHR_chunk = IDHR_chunk_opt.ok_or("IDHR chunk not found")?; //Code to find the IDHR chunk from our chunk list
        let color_type = &IDHR_chunk.color_type; //Will always have a happy path

        let color = match color_type {
            ColorType::Grayscale | ColorType::GrayscaleAlpha => {
                let gray = png_file.big_endian_u16()?;
                Color::Gray(gray)
            }
            ColorType::RGB | ColorType::RGBA => {
                let red = png_file.big_endian_u16()?;
                let green = png_file.big_endian_u16()?;
                let blue = png_file.big_endian_u16()?;
                Color::RGB(red, green, blue)
            }
            ColorType::Indexed => {
                let palette_index = png_file.get_u8()?;
                Color::PaletteIndex(palette_index)
            }
        };

        let CRC = png_file.get_u32()?;

        Ok(Self{length, color, CRC})
    }
}

#[derive(Debug)]
enum Color {
    Gray(u16),
    RGB(u16, u16, u16),
    PaletteIndex(u8),
}

#[derive(Debug)]
pub enum BackgroundColor {
    PaletteIndex(u8),
    Grayscale(u16),
    RGB(u16, u16, u16),
}

//Gama chunk
#[derive(Debug)]
pub struct gAMAChunk {
    length: u32,
    gamma: u32,
    CRC: Vec<u8>,
}

impl gAMAChunk {
    pub fn new(length: u32, png_file: &mut Png) -> Result<Self, Box<dyn Error>> {
        let gamma = png_file.big_endian_u32()?;

        let CRC = png_file.get_u32()?;

        Ok(Self{length, gamma, CRC})
    }
}

//cHRM chunk

#[derive(Debug)]
pub struct cHRMChunk {
    length: u32,
    white_point_x: u32,
    white_point_y: u32,
    red_x: u32,
    red_y: u32,
    green_x: u32,
    green_y: u32,
    blue_x: u32,
    blue_y: u32,
    CRC: Vec<u8>,
}

impl cHRMChunk {
    pub fn new(length: u32, png_file: &mut Png) -> Result<Self, Box<dyn Error>> {
        let white_point_x = png_file.big_endian_u32()?;
        let white_point_y = png_file.big_endian_u32()?;
        let red_x = png_file.big_endian_u32()?;
        let red_y = png_file.big_endian_u32()?;
        let green_x = png_file.big_endian_u32()?;
        let green_y = png_file.big_endian_u32()?;
        let blue_x = png_file.big_endian_u32()?;
        let blue_y = png_file.big_endian_u32()?;
        let CRC = png_file.get_u32()?;

        Ok(Self {length, white_point_x, white_point_y, red_x, red_y, green_x, green_y, blue_x, blue_y, CRC})
    }
}

//dSIG 
#[derive(Debug)]
pub struct dSIGChunk {
    length: u32,
    data: Vec<u8>,
    CRC: Vec<u8>,
}

impl dSIGChunk {
    pub fn new(length: u32, png_file: &mut Png) -> Result<Self, Box<dyn Error>> {
        let mut data = Vec::new();

        for _ in 0..length {
            data.push(png_file.get_u8()?);
        }

        let CRC = png_file.get_u32()?;

        Ok(Self{length, data, CRC})
    }
}

// eXIf chunk
#[derive(Debug)]
pub struct eXIfChunk {
    length: u32,
    data: Vec<u8>,
    CRC: Vec<u8>,
}

impl eXIfChunk {
    pub fn new(length: u32, png_file: &mut Png) -> Result<Self, Box<dyn Error>> {
        let mut data = Vec::new();

        for _ in 0..length {
            data.push(png_file.get_u8()?);
        }

        let CRC = png_file.get_u32()?;

        Ok(Self{length, data, CRC})
    }
}

// hIST chunk
#[derive(Debug)]
pub struct hISTChunk {
    length: u32,
    data: Vec<u16>,
    CRC: Vec<u8>,
}

impl hISTChunk {
    pub fn new(length: u32, png_file: &mut Png) -> Result<Self, Box<dyn Error>> {
        let mut data = Vec::new();

        for _ in 0..length/2 {
            data.push(png_file.big_endian_u16()?);
        }

        let CRC = png_file.get_u32()?;

        Ok(Self{length, data, CRC})
    }
}


#[derive(Debug)]
pub struct iCCPChunk {
    length: u32,
    profile_name: String,
    compression_method: u8,
    compression_profile: Vec<u8>,
    CRC: Vec<u8>,
}

impl iCCPChunk {
    pub fn new(length: u32, png_file: &mut Png) -> Result<Self, Box<dyn Error>> {
        let (profile_name, profile_name_length) = png_file.read_null_terminated_string()?;
        let compression_method = png_file.get_u8()?;
        let compression_profile = png_file.read_bytes((length - profile_name_length - 1) as usize)?;

        let CRC = png_file.get_u32()?;

        Ok(Self {length, profile_name, compression_method, compression_profile, CRC})
    }
}

//iTxtChunk
#[derive(Debug)]
pub struct iTXtChunk {
    length: u32,
    keyword: String,
    compression_flag: u8,
    compression_method: u8,
    language_tag: String,
    translated_keyword: String,
    text: String,
    CRC: Vec<u8>,
}

impl iTXtChunk {
    pub fn new(length: u32, png_file: &mut Png) -> Result<Self, Box<dyn Error>> {
        let (keyword, keyword_length) = png_file.read_null_terminated_string()?;

        let compression_flag = png_file.get_u8()?;
        let compression_method = png_file.get_u8()?;

        let (language_tag, language_tag_length) = png_file.read_null_terminated_string()?;

        let (translated_keyword, translated_keyword_length) = png_file.read_null_terminated_string()?;

        let mut text_bytes = Vec::new();
        for _ in 0..(length - keyword_length - language_tag_length - translated_keyword_length - 2) {
            text_bytes.push(png_file.get_u8()?);
        }
        let text = String::from_utf8(text_bytes)?;

        let CRC = png_file.get_u32()?;

        Ok(Self{length, keyword, compression_flag, compression_method, language_tag, translated_keyword, text, CRC})
    }
}

//pHYs Chunk
#[derive(Debug)]
pub struct pHYsChunk {
    length: u32,
    pixels_per_unit_x_axis: u32,
    pixels_per_unit_y_axis: u32,
    unit_specifier: u8,
    CRC: Vec<u8>,
}

impl pHYsChunk {
    pub fn new(length: u32, png_file: &mut Png) -> Result<Self, Box<dyn Error>> {
        let pixels_per_unit_x_axis = png_file.big_endian_u32()?;
        let pixels_per_unit_y_axis = png_file.big_endian_u32()?;
        let unit_specifier = png_file.get_u8()?;
        let CRC = png_file.get_u32()?;

        Ok(Self{length, pixels_per_unit_x_axis, pixels_per_unit_y_axis, unit_specifier, CRC})
    }
}

//sBIT
#[derive(Debug)]
pub struct sBITChunk {
    length: u32,
    data: Vec<u8>,
    CRC: Vec<u8>,
}

impl sBITChunk {
    pub fn new(length: u32, png_file: &mut Png) -> Result<Self, Box<dyn Error>> {
        let mut data = Vec::new();

        for _ in 0..length {
            data.push(png_file.get_u8()?);
        }

        let CRC = png_file.get_u32()?;

        Ok(Self{length, data, CRC})
    }
}

//sPLT Chunk (I hate these useless chunks)

//sPLT is very similar to PLTE
#[derive(Debug)]
pub struct sPLTEntry {
    red: u16,
    green: u16,
    blue: u16,
    alpha: u16,
    frequency: u32,
}


#[derive(Debug)]
pub struct sPLTChunk {
    length: u32,
    palette_name: String,
    sample_depth: u8,
    entries: Vec<sPLTEntry>,
    CRC: Vec<u8>,
}

impl sPLTChunk {
    pub fn new(length: u32, png_file: &mut Png) -> Result<Self, Box<dyn Error>> {
        let (palette_name, name_length) = png_file.read_null_terminated_string()?;
        let sample_depth = png_file.get_u8()?;

        let mut entries = Vec::new();
        let entry_length = if sample_depth == 8 { 6 } else { 10 };
        let num_entries = (length - name_length - 2) / entry_length;

        for _ in 0..num_entries { //We have to adjust for the sample depth value
            let red = if sample_depth == 8 { png_file.get_u8()? as u16 } else { png_file.big_endian_u16()? };
            let green = if sample_depth == 8 { png_file.get_u8()? as u16 } else { png_file.big_endian_u16()? };
            let blue = if sample_depth == 8 { png_file.get_u8()? as u16 } else { png_file.big_endian_u16()? };
            let alpha = if sample_depth == 8 { png_file.get_u8()? as u16 } else { png_file.big_endian_u16()? };
            let frequency = png_file.big_endian_u32()?;

            entries.push(sPLTEntry { red, green, blue, alpha, frequency });
        }

        let CRC = png_file.get_u32()?;

        Ok(Self{length, palette_name, sample_depth, entries, CRC})
    }
}

/* sRGB
The following values are defined for the rendering intent:
0: Perceptual
1: Relative colorimetric
2: Saturation
3: Absolute colorimetric*/

#[derive(Debug)]
pub enum RenderingIntent {
    Perceptual,
    RelativeColorimetric,
    Saturation,
    AbsoluteColorimetric,
}

#[derive(Debug)]
pub struct sRGBChunk {
    length: u32,
    rendering_intent: RenderingIntent,
    CRC: Vec<u8>,
}

impl sRGBChunk {
    pub fn new(length: u32, png_file: &mut Png) -> Result<Self, Box<dyn Error>> {
        let rendering_intent = match png_file.get_u8()? {
            0 => RenderingIntent::Perceptual,
            1 => RenderingIntent::RelativeColorimetric,
            2 => RenderingIntent::Saturation,
            3 => RenderingIntent::AbsoluteColorimetric,
            _ => return Err("Invalid value for rendering intent".into()),
        };
        let CRC = png_file.get_u32()?;

        Ok(Self { length, rendering_intent, CRC })
    }
}

// sTER extremely odd chunk which has little doccumentation but listed on wikipedia so I've decided to implement it
#[derive(Debug)]
pub struct sTERChunk {
    length: u32,
    stereo_mode: u8,
    CRC: Vec<u8>,
}

impl sTERChunk {
    pub fn new(length: u32, png_file: &mut Png) -> Result<Self, Box<dyn Error>> {
        let stereo_mode = png_file.get_u8()?;
        let CRC = png_file.get_u32()?;

        Ok(Self { length, stereo_mode, CRC })
    }
}

//tEXt Chunk some improvements need to be made here
#[derive(Debug)]
pub struct tEXtChunk {
    length: u32,
    keyword: String,
    text: String,
    CRC: Vec<u8>,
}

impl tEXtChunk {
    pub fn new(length: u32, png_file: &mut Png) -> Result<Self, Box<dyn Error>> {
        let (keyword, keyword_length) = png_file.read_null_terminated_string()?;
        let text_length = length - keyword_length - 1; // Subtract 1 for the null character
        let text = png_file.get_string(text_length as usize)?;
        let CRC = png_file.get_u32()?;

        Ok(Self { length, keyword, text, CRC })
    }
}

//tIME Chunk
#[derive(Debug)]
pub struct tIMEChunk {
    length: u32,
    year: u16,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    CRC: Vec<u8>,
}

impl tIMEChunk {
    pub fn new(length: u32, png_file: &mut Png) -> Result<Self, Box<dyn Error>> {
        let year = png_file.big_endian_u16()?;
        let month = png_file.get_u8()?;
        let day = png_file.get_u8()?;
        let hour = png_file.get_u8()?;
        let minute = png_file.get_u8()?;
        let second = png_file.get_u8()?;
        let CRC = png_file.get_u32()?;

        Ok(Self{length, year, month, day, hour, minute, second, CRC})
    }
}

//tRNS chunk
#[derive(Debug)]
pub struct tRNSChunk {
    length: u32,
    transparency_data: Vec<u8>,
    CRC: Vec<u8>,
}

impl tRNSChunk {
    pub fn new(length: u32, png_file: &mut Png) -> Result<Self, Box<dyn Error>> {
        let mut transparency_data = Vec::new();
        for _ in 0..length {
            transparency_data.push(png_file.get_u8()?);
        }
        let CRC = png_file.get_u32()?;

        Ok(Self { length, transparency_data, CRC })
    }
}


//Compressed Text chunk zTXt
#[derive(Debug)]
pub struct zTXtChunk {
    length: u32,
    keyword: String,
    compression_method: u8,
    compressed_text: Vec<u8>,
    CRC: Vec<u8>,
}

impl zTXtChunk {
    pub fn new(length: u32, png_file: &mut Png) -> Result<Self, Box<dyn Error>> {
        let mut keyword = String::new();
        //I won't declare a function for this as it's only used once
        loop {
            let c = png_file.get_u8()? as char;
            if c == '\0' {
                break;
            }
            keyword.push(c);
        }
        let compression_method = png_file.get_u8()?;
        let mut compressed_text = Vec::new();
        for _ in 0..length - keyword.len() as u32 - 2 {
            compressed_text.push(png_file.get_u8()?);
        }
        let CRC = png_file.get_u32()?;

        Ok(Self{length, keyword, compression_method, compressed_text, CRC})
    }
}

/*With this code I have to implement every type of chunk because I am sequentially reading it. However I very well could move the sequential counter forward based
off chunnk length to avoid reading some chunks which are not needed for decoding, for education I've decided to implement every chunk */
#[derive(Debug)]
pub enum Chunk {
    IDHR(IDHRChunk),
    PLTE(PLTEChunk),
    IDAT(IDATChunk),
    IEND(IENDChunk),
    tIME(tIMEChunk),
    bKGD(bKGDChunk),
    gAMA(gAMAChunk),
    cHRM(cHRMChunk),
    dSIG(dSIGChunk),
    eXIf(eXIfChunk),
    hIST(hISTChunk),
    iCCP(iCCPChunk),
    iTXt(iTXtChunk),
    pHYs(pHYsChunk),
    sBIT(sBITChunk),
    sPLT(sPLTChunk),
    sRGB(sRGBChunk),
    sTER(sTERChunk),
    tEXt(tEXtChunk),
    tRNS(tRNSChunk),
    zTXt(zTXtChunk),
}
