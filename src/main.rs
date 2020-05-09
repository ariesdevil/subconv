use argh::FromArgs;
use subconv::convert::conv;
use walkdir::WalkDir;

#[argh(description = "convert subtitle files(vtt and srt) bi-direction")]
#[derive(FromArgs, Debug)]
struct SubConvArgs {
    #[argh(option, short = 'd', description = "dirs of subtitles resident")]
    dirs: Vec<String>,

    #[argh(option, short = 'f', description = "file path")]
    file: Option<String>,
}

fn main() {
    let args: SubConvArgs = argh::from_env();
    match args.file {
        Some(f) => conv::convert(&f, conv::Direction::VTT2SRT),
        None => Ok(()),
    };
    for dir in args.dirs {
        let walker = WalkDir::new(dir).into_iter();
        for entry in walker.filter_map(|e| e.ok()) {
            let f_name = entry
                .path()
                .to_str()
                .expect(&format!("wrong file path: {}", entry.path().display()));
            if f_name.ends_with(".vtt") {
                println!("{}", f_name);
                match conv::convert(f_name, conv::Direction::VTT2SRT) {
                    Ok(_) => println!("Success"),
                    Err(e) => panic!(e),
                }
            }
        }
    }
}
