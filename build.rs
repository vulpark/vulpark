// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{fs, path::Path};

fn main() {
    #[cfg(windows)]
    const LINE_ENDING: &'static str = "\r\n";
    #[cfg(not(windows))]
    const LINE_ENDING: &'static str = "\n";
    
    let header = include_str!("./license_header")
        .split(LINE_ENDING)
        .map(|it| format!("// {it}{LINE_ENDING}"))
        .reduce(|acc, line| format!("{acc}{line}"))
        .unwrap();
    let _ = visit_files(Path::new("src"), &header);
}

fn visit_files(path: &Path, header: &str) -> Result<(), std::io::Error> {
    if path.is_file() {
        return write_header(path, header);
    }
    for new in path.read_dir().unwrap() {
        visit_files(new?.path().as_path(), header).unwrap();
    }
    Ok(())
}

fn write_header(path: &Path, header: &str) -> Result<(), std::io::Error> {
    let Some(ext) = path.extension() else {
        return Ok(())
    };
    if ext != "rs" {
        return Ok(());
    }
    let mut file = fs::read_to_string(path)?;
    if !file.starts_with(header) {
        let mut tmp = header.to_string();
        tmp.push_str(&format!("{file}\n"));
        file = tmp;
    }
    fs::write(path, file)?;
    Ok(())
}
