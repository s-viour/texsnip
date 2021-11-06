#[macro_use]
extern crate anyhow;
extern crate dirs;

use anyhow::{Context, Result};
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::process::Command;

const TEMPLATE_STR_START: &'static str = r#"
\documentclass{article}
\usepackage{amsmath}
\usepackage{amsthm}
\usepackage{amssymb}
\usepackage{bm}
\usepackage[active,displaymath,textmath,tightpage]{preview}
\pagestyle{empty}
\begin{document}
\begin{equation*}
"#;

const TEMPLATE_STR_END: &'static str = r#"\end{equation*}
\end{document}
"#;

fn main() -> Result<()> {
    let tempdir = env::temp_dir();
    let working = tempdir.join("texsnip");
    fs::create_dir_all(&working)?;
    env::set_current_dir(&working).context("failed to change into working dir")?;

    remove_extrafiles();
    write_texfile()?;
    // latex -halt-on-error -interaction batchmode -output-directory /bar /bar/foo.tex
    let status = Command::new("latex")
        .args(["-halt-on-error", "-interaction", "batchmode", "input.tex"])
        .spawn()?
        .wait()?;
    if !status.success() {
        remove_extrafiles();
        bail!("latex failed: {}", status);
    }

    // dvipng -D 225 -z 9 -bg white -o /bar/foo.png /bar/foo.dvi
    let status = Command::new("dvipng")
        .args(["-D", "512", "-z", "9", "-o", "out.png", "input.dvi"])
        .spawn()?
        .wait()?;
    if !status.success() {
        remove_extrafiles();
        bail!("dvipng failed: {}", status);
    }

    // magick convert out.png -trim +repage -bordercolor White -border 64x64 result.png
    let status = Command::new("magick")
        .args([
            "convert",
            "out.png",
            "-trim",
            "+repage",
            "-bordercolor",
            "White",
            "-border",
            "64x64",
            "result.png",
        ])
        .spawn()?
        .wait()?;
    if !status.success() {
        remove_extrafiles();
        bail!("convert failed: {}", status);
    }

    Ok(())
}

fn remove_extrafiles() {
    fs::remove_file("input.tex").ok();
    fs::remove_file("input.dvi").ok();
    fs::remove_file("out.png").ok();
    fs::remove_file("result.png").ok();
}

fn write_texfile() -> Result<()> {
    let mut input = Vec::new();
    io::stdin().read_to_end(&mut input)?;
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("input.tex")?;

    file.write_all(TEMPLATE_STR_START.as_bytes())?;
    file.write_all(&input)?;
    file.write_all(TEMPLATE_STR_END.as_bytes())?;
    Ok(())
}
