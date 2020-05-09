use regex::Regex;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use voca_rs::strip;

#[cfg(windows)]
const EOL: &'static str = "\r\n";
#[cfg(not(windows))]
const EOL: &'static str = "\n";

trait Split<T> {
    fn split(&self) -> Vec<String>;
}

pub enum Direction {
    VTT2SRT,
    SRT2VTT,
}

pub fn convert(file: &str, direct: Direction) -> std::io::Result<()> {
    match direct {
        Direction::VTT2SRT => {
            let content = vtt2srt(file)?;
            let out_path = format!("{}.{}", file.strip_suffix(".vtt").unwrap(), "srt");
            write_content(out_path, content)?
        }
        Direction::SRT2VTT => unimplemented!(),
    }

    Ok(())
}

fn read_content<T: AsRef<Path>>(file: T) -> std::io::Result<String> {
    let mut f = File::open(file)?;

    let mut s = String::new();
    f.read_to_string(&mut s)?;
    Ok(s)
}

fn write_content<T: AsRef<Path>>(file: T, content: String) -> std::io::Result<()> {
    let mut f = if !file.as_ref().exists() {
        let out_path = Path::new(file.as_ref());
        File::create(out_path)?
    } else {
        OpenOptions::new().write(true).open(file)?
    };
    f.write_all(content.as_bytes())
}

impl Split<Direction> for String {
    fn split(&self) -> Vec<String> {
        self.lines().map(String::from).collect()
    }
}

fn remove_header(lines: Vec<String>) -> Vec<String> {
    lines
        .into_iter()
        .skip_while(move |line| line.trim().ne(""))
        .collect()
}

pub fn vtt2srt(file: &str) -> std::io::Result<String> {
    let content = read_content(file)?;
    let lines = content.split();
    let mut lines: Vec<String> = remove_header(lines);

    lazy_static! {
        static ref PATTERN1: Regex =
            Regex::new(r"(?P<H>\d{2}):(?P<M>\d{2}):(?P<S>\d{2})\.(?P<f>\d{3})").unwrap();
        static ref PATTERN1_START: Regex = Regex::new(r"^\d{2}:\d{2}:\d{2}\.\d{3}").unwrap();
        static ref PATTERN2: Regex =
            Regex::new(r"(?P<M>\d{2}):(?P<S>\d{2})\.(?P<f>\d{3})").unwrap();
        static ref PATTERN2_START: Regex = Regex::new(r"^\d{2}:\d{2}\.\d{3}").unwrap();

        // Well, it seems that the time line is far more complex than we think before.
        // For example: 00:00:00.060 --> 00:00:04.490 align:start position:0%
        // so we trim the modifiers after time line
        static ref MODIFY_PATTERN: Regex = Regex::new(r" [a-zA-Z]\w*:\S+").unwrap();
    }

    let mut output = String::new();
    let mut n = 0;
    for line in lines.iter_mut() {
        if PATTERN1_START.is_match(line) {
            n = n + 1;
            output = format!("{}{}{}", output, EOL, n); // Add an empty line, to make Aegisub happy.
            output = format!("{}{}", output, EOL);
            *line = PATTERN1.replace_all(line, "$H:$M:$S,$f").to_string();
            *line = MODIFY_PATTERN.replace_all(line, "").to_string();
        } else if PATTERN2_START.is_match(line) {
            n = n + 1;
            output = format!("{}{}{}", output, EOL, n); // Add an empty line, to make Aegisub happy.
            output = format!("{}{}", output, EOL);
            *line = PATTERN2.replace_all(line, "00:$M:$S,$f").to_string();
            *line = MODIFY_PATTERN.replace_all(line, "").to_string();
        }
        output = format!("{}{}{}", output, strip::strip_tags(line), EOL);
    }
    Ok(output.trim().to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_vtt2srt_tmp() {
        let bytes = include_bytes!("../../resources/demo.vtt");
        let mut file_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        file_path.push("resources/demo.vtt");
        assert_eq!(
            String::from_utf8_lossy(bytes),
            read_content(file_path.as_path()).unwrap()
        );
    }

    #[test]
    fn test_vtt2srt() {
        let base = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
        let vtt_file_path = format!("{}/{}", base, "resources/demo.vtt");

        let srt_content = String::from_utf8_lossy(include_bytes!("../../resources/demo.srt"));
        let srt_file_path = format!("{}/{}", base, "resources/demo.srt");
        let convert_srt_content = vtt2srt(&vtt_file_path).unwrap();
        println!("{}", convert_srt_content);
        let out_path = Path::new("/Users/ariesdevil/Downloads/01.srt");
        let mut out_file = File::create(&out_path).unwrap();
        out_file.write_all(convert_srt_content.as_bytes()).unwrap();
        assert_eq!(srt_content, convert_srt_content);
    }
}
