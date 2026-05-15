use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::fmt;
use std::fs;
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum Format {
    Elf,
    Pe,
    MachO,
    IntelHex,
    Bin,
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Format::Elf => "ELF",
            Format::Pe => "PE",
            Format::MachO => "Mach-O",
            Format::IntelHex => "Intel HEX",
            Format::Bin => "BIN",
        };
        write!(f, "{}", name)
    }
}

impl Format {
    pub fn detect(bytes: &[u8]) -> Self {
        if bytes.len() >= 4 && bytes[0..4] == [0x7F, b'E', b'L', b'F'] {
            return Format::Elf;
        }

        if let Some(format) = detect_pe(bytes) {
            return format;
        }

        if let Some(format) = detect_macho(bytes) {
            return format;
        }

        if detect_intel_hex(bytes) {
            return Format::IntelHex;
        }

        Format::Bin
    }
}

fn detect_pe(bytes: &[u8]) -> Option<Format> {
    if bytes.len() < 0x40 {
        return None;
    }
    if &bytes[0..2] != b"MZ" {
        return None;
    }
    let pe_header_offset = u32::from_le_bytes([bytes[0x3C], bytes[0x3D], bytes[0x3E], bytes[0x3F]]) as usize;
    if pe_header_offset + 4 <= bytes.len() && &bytes[pe_header_offset..pe_header_offset + 4] == b"PE\0\0" {
        Some(Format::Pe)
    } else {
        None
    }
}

fn detect_macho(bytes: &[u8]) -> Option<Format> {
    if bytes.len() < 4 {
        return None;
    }
    let magic = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    match magic {
        0xFEED_FACE | 0xCEFA_EDFE | 0xFEED_FACF | 0xCFFA_EDFE => Some(Format::MachO),
        _ => None,
    }
}

fn detect_intel_hex(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return false;
    }
    let text = match std::str::from_utf8(bytes) {
        Ok(v) => v,
        Err(_) => return false,
    };
    text.lines().all(|line| line.is_empty() || line.starts_with(':'))
}

#[derive(Deserialize, Debug)]
pub struct FormatConfig {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub endianness: Option<String>,
    #[serde(default)]
    pub magic: Option<Vec<u8>>,
    #[serde(default)]
    pub header_size: Option<usize>,
    #[serde(default)]
    pub header_fields: Option<serde_json::Value>,
    #[serde(default)]
    pub sections: Option<serde_json::Value>,
    #[serde(default)]
    pub validators: Option<serde_json::Value>,
    #[serde(default)]
    pub transformers: Option<serde_json::Value>,
}

impl FormatConfig {
    pub fn load(path: &Path) -> Result<Self> {
        let bytes = fs::read(path).with_context(|| format!("Failed to read config file: {}", path.display()))?;
        let config: FormatConfig = serde_json::from_slice(&bytes).with_context(|| format!("Failed to parse JSON config: {}", path.display()))?;
        Ok(config)
    }
}

pub fn convert_format(from: Format, to: Format, input_path: &Path, output_path: &Path, remove_input: bool) -> Result<()> {
    let data = fs::read(input_path).with_context(|| format!("Failed to read input file: {}", input_path.display()))?;
    let actual = Format::detect(&data);
    if actual != from {
        bail!("Input file format mismatch: expected {} but detected {}", from, actual);
    }

    let output = if from == to {
        data
    } else {
        match (from, to) {
            (Format::Bin, Format::IntelHex) => encode_intel_hex(&data).into_bytes(),
            (Format::IntelHex, Format::Bin) => parse_intel_hex(&data)?,
            _ => bail!("Conversion from {} to {} is not supported yet", from, to),
        }
    };

    fs::write(output_path, &output).with_context(|| format!("Failed to write output file: {}", output_path.display()))?;
    if remove_input {
        fs::remove_file(input_path).with_context(|| format!("Failed to remove input file: {}", input_path.display()))?;
    }
    Ok(())
}

pub fn run_config_conversion(config_path: &Path, input_path: &Path, output_path: &Path, convert_to_format: bool, remove_input: bool) -> Result<()> {
    let config = FormatConfig::load(config_path)?;
    let input_bytes = fs::read(input_path).with_context(|| format!("Failed to read input file: {}", input_path.display()))?;

    let magic = config.magic.clone().unwrap_or_default();
    let header_size = config.header_size.unwrap_or(magic.len());

    let output_bytes = if convert_to_format {
        let mut buffer = Vec::new();
        buffer.extend(&magic);
        if header_size > magic.len() {
            buffer.resize(header_size, 0);
        }
        buffer.extend(&input_bytes);
        buffer
    } else {
        if !magic.is_empty() && input_bytes.len() < magic.len() {
            bail!("Input file is smaller than the expected magic size for the config format");
        }
        if !magic.is_empty() && input_bytes.starts_with(&magic) {
            let payload_start = header_size.min(input_bytes.len());
            input_bytes[payload_start..].to_vec()
        } else if !magic.is_empty() {
            bail!("Input file does not match the expected magic header for config format {}", config.name);
        } else {
            input_bytes
        }
    };

    fs::write(output_path, &output_bytes).with_context(|| format!("Failed to write output file: {}", output_path.display()))?;
    if remove_input {
        fs::remove_file(input_path).with_context(|| format!("Failed to remove input file: {}", input_path.display()))?;
    }
    Ok(())
}

fn encode_intel_hex(data: &[u8]) -> String {
    let mut result = String::new();
    let mut address = 0u32;
    for chunk in data.chunks(16) {
        let len = chunk.len() as u8;
        let record_type = 0u8;
        let checksum = checksum_intel_hex(len, address as u16, record_type, chunk);
        result.push(':');
        result.push_str(&format!("{:02X}{:04X}{:02X}", len, address as u16, record_type));
        for byte in chunk {
            result.push_str(&format!("{:02X}", byte));
        }
        result.push_str(&format!("{:02X}\n", checksum));
        address += len as u32;
    }
    result.push_str(":00000001FF\n");
    result
}

fn checksum_intel_hex(len: u8, address: u16, record_type: u8, data: &[u8]) -> u8 {
    let mut sum = len as u32;
    sum += ((address >> 8) & 0xFF) as u32;
    sum += (address & 0xFF) as u32;
    sum += record_type as u32;
    for byte in data {
        sum += *byte as u32;
    }
    ((!sum + 1) & 0xFF) as u8
}

fn parse_intel_hex(bytes: &[u8]) -> Result<Vec<u8>> {
    let text = std::str::from_utf8(bytes).context("Intel HEX file is not valid UTF-8")?;
    let mut output = Vec::new();
    for (line_index, raw_line) in text.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        if !line.starts_with(':') {
            bail!("Intel HEX parse error on line {}: missing ':'", line_index + 1);
        }
        let record = &line[1..];
        if record.len() < 10 {
            bail!("Intel HEX parse error on line {}: too short", line_index + 1);
        }
        let count = u8::from_str_radix(&record[0..2], 16).context("Failed to parse byte count")?;
        let address = u16::from_str_radix(&record[2..6], 16).context("Failed to parse address")?;
        let record_type = u8::from_str_radix(&record[6..8], 16).context("Failed to parse record type")?;
        let data_end = 8 + (count as usize) * 2;
        if record.len() < data_end + 2 {
            bail!("Intel HEX parse error on line {}: data length mismatch", line_index + 1);
        }
        let data_str = &record[8..data_end];
        let checksum = u8::from_str_radix(&record[data_end..data_end + 2], 16).context("Failed to parse checksum")?;
        let mut record_data = Vec::with_capacity(count as usize);
        for chunk in data_str.as_bytes().chunks(2) {
            let byte_str = std::str::from_utf8(chunk).context("Intel HEX contains invalid byte characters")?;
            record_data.push(u8::from_str_radix(byte_str, 16).context("Failed to parse data byte")?);
        }
        let calculated = checksum_intel_hex(count, address, record_type, &record_data);
        if calculated != checksum {
            bail!("Intel HEX checksum mismatch on line {}", line_index + 1);
        }
        match record_type {
            0 => {
                let end = (address as usize) + record_data.len();
                if output.len() < end {
                    output.resize(end, 0);
                }
                output[(address as usize)..end].copy_from_slice(&record_data);
            }
            1 => break,
            _ => (),
        }
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intel_hex_roundtrip() {
        let data = b"Hello, FEX!";
        let hex = encode_intel_hex(data);
        let roundtrip = parse_intel_hex(hex.as_bytes()).expect("Parse should succeed");
        assert_eq!(roundtrip, data);
    }

    #[test]
    fn test_detect_elf() {
        let bytes = [0x7F, b'E', b'L', b'F', 0, 0, 0, 0];
        assert_eq!(Format::detect(&bytes), Format::Elf);
    }
}
