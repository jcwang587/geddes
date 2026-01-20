use crate::error::GeddesError;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use zip::ZipArchive;

#[derive(Debug)]
pub struct ParsedData {
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub e: Option<Vec<f64>>,
}

pub fn parse_xy(path: &Path) -> Result<ParsedData, GeddesError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut x = Vec::new();
    let mut y = Vec::new();
    let mut e = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with('!') {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            if let (Ok(val_x), Ok(val_y)) = (parts[0].parse::<f64>(), parts[1].parse::<f64>()) {
                x.push(val_x);
                y.push(val_y);
                if parts.len() >= 3 {
                     if let Ok(val_e) = parts[2].parse::<f64>() {
                         e.push(val_e);
                     }
                }
            }
        }
    }
    
    let has_error = !e.is_empty() && e.len() == x.len();
    Ok(ParsedData { 
        x, 
        y, 
        e: if has_error { Some(e) } else { None } 
    })
}

pub fn parse_rasx(path: &Path) -> Result<ParsedData, GeddesError> {
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file)?;
    
    let names: Vec<String> = (0..archive.len())
        .filter_map(|i| archive.by_index(i).ok().map(|f| f.name().to_string()))
        .collect();
        
    // Prioritize Data0/Profile0.txt, or find any Profile*.txt
    let profile_name = names.iter()
        .find(|n| n.contains("Profile") && n.ends_with(".txt"))
        .ok_or_else(|| GeddesError::FileNotFoundInArchive("Profile*.txt".to_string()))?;

    let mut file = archive.by_name(profile_name)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    
    let mut x = Vec::new();
    let mut y = Vec::new();
    
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
             if let (Ok(val_x), Ok(val_y)) = (parts[0].parse::<f64>(), parts[1].parse::<f64>()) {
                x.push(val_x);
                y.push(val_y);
            }
        }
    }
     Ok(ParsedData { x, y, e: None })
}

pub fn parse_raw(path: &Path) -> Result<ParsedData, GeddesError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    
    let mut start = 0.0;
    let mut step = 0.0;
    
    let mut header_found = false;
    
    while let Some(line_res) = lines.next() {
        let line = line_res?;
        if line.starts_with("BANK") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            // BANK 1 4941 494 CONST 1600.0 1.7 0.0 0.0 STD
            if parts.len() >= 7 {
                let start_raw = parts[5].parse::<f64>().map_err(|_| GeddesError::Parse("Invalid start".into()))?;
                let step_raw = parts[6].parse::<f64>().map_err(|_| GeddesError::Parse("Invalid step".into()))?;
                
                // GSAS standard: centidegrees
                start = start_raw / 100.0;
                step = step_raw / 100.0;
                header_found = true;
                break;
            }
        }
    }
    
    if !header_found {
        return Err(GeddesError::Parse("BANK header not found in RAW file".into()));
    }
    
    let mut y = Vec::new();
    
    for line in lines {
        let line = line?;
        let parts = line.split_whitespace();
        for part in parts {
            if let Ok(val) = part.parse::<f64>() {
                y.push(val);
            }
        }
    }
    
    // Generate x
    let mut x = Vec::with_capacity(y.len());
    for i in 0..y.len() {
        x.push(start + (i as f64) * step);
    }
    
    Ok(ParsedData { x, y, e: None })
}
