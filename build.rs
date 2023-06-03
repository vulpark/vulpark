// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{fs, path::Path};

fn main() {
    let mut header = include_str!("./license_header").split("\n").map(|it| format!("// {it}\n") ).reduce(|acc, line| format!("{acc}{line}") ).unwrap();
    header.push('\n');
    let _ = visit_files(Path::new("src"), &header);
}

fn visit_files(path: &Path, header: &str) -> Result<(), std::io::Error> {
    if path.is_file() {
        return write_header(path, header)
    }
    for new in path.read_dir().unwrap() {
        visit_files(new?.path().as_path(), header).unwrap()
    }
    Ok(())
}

fn write_header(path: &Path, header: &str) -> Result<(), std::io::Error> {
    let Some(ext) = path.extension() else {
        return Ok(())
    };
    if ext != "rs" {
        return Ok(())
    }
    let mut text = fs::read_to_string(path)?;
    if !text.starts_with(header) {
        let mut tmp = header.to_string();
        tmp.push_str(&text);
        text = tmp;
    }
    let _ = fs::write(path, text)?;
    Ok(())
}
