const START_TEX_FILE: &str = r"
\documentclass{standalone}

\usepackage{amsmath}
\usepackage{amsfonts}
\usepackage{amssymb}
\usepackage{tikz-cd}
\usetikzlibrary{quantikz}
\usetikzlibrary{decorations.pathmorphing}

\newcommand{\NN}{\mathbb{N}}
\newcommand{\ZZ}{\mathbb{Z}}
\newcommand{\QQ}{\mathbb{Q}}
\newcommand{\RR}{\mathbb{R}}
\newcommand{\CC}{\mathbb{C}}
\newcommand{\PP}{\mathbb{P}}
\renewcommand{\AA}{\mathbb{A}}
\newcommand{\id}{\textup{id}}
\newcommand{\bdot}{\bullet}
\newcommand{\isom}{\cong}
\DeclareMathOperator{\Spec}{Spec}
\DeclareMathOperator{\coker}{coker}
\DeclareMathOperator{\Hom}{Hom}
\DeclareMathOperator{\iHom}{\underline{Hom}}

\begin{document}
";

const END_TEX_FILE: &str = r"
\end{document}
";

const TMP_DIRECTORY: &str = "tmp";

use std::{
    fs::{self, File},
    hash::{DefaultHasher, Hasher},
    io::Write,
    path::Path,
    process::Command,
};

pub struct TexSvg {
    file: File,
    hasher: DefaultHasher,
}

impl TexSvg {
    pub fn new() -> Self {
        // Create temporary file for writing in temporary directory
        fs::create_dir_all(TMP_DIRECTORY).unwrap();
        let mut file = File::create(&format!("{TMP_DIRECTORY}/tmp.tex")).unwrap();

        // Write start of tex document to file
        file.write(START_TEX_FILE.as_bytes()).unwrap();

        Self {
            file,
            hasher: DefaultHasher::new(),
        }
    }

    pub fn feed(&mut self, data: &str) -> std::io::Result<()> {
        // Write data to file
        self.file.write(data.as_bytes())?;

        // Feed data to hasher
        self.hasher.write(data.as_bytes());

        Ok(())
    }

    pub fn compile(mut self, target_directory: &str) -> Result<String, String> {
        // Write end of tex document to file
        self.file
            .write(END_TEX_FILE.as_bytes())
            .or(Err("Failed to write to file"))?;

        // Compute svg path from hash
        let hash = self.hasher.finish();
        let svg_filename = format!("{hash}.svg");
        let svg_path = format!("{target_directory}/{svg_filename}");

        // Close file by dropping self
        drop(self);

        // Compile to svg only if svg does not yet exist
        if !Path::new(&svg_path).exists() {
            match Command::new("pdflatex")
                .args([
                    "-halt-on-error",
                    "-interaction=nonstopmode",
                    &format!("-output-directory={TMP_DIRECTORY}"),
                    &format!("{TMP_DIRECTORY}/tmp.tex"),
                ])
                .output()
            {
                Ok(output) => {
                    if output.status.code() != Some(0) {
                        return Err(format!(
                            "Failed to run `pdflatex`: {}",
                            String::from_utf8_lossy(&output.stdout)
                        ));
                    }
                }
                Err(err) => {
                    return Err(format!("Failed to run `pdflatex`: {err:?}"));
                }
            }

            match Command::new("pdf2svg")
                .args([&format!("{TMP_DIRECTORY}/tmp.pdf"), &svg_path])
                .output()
            {
                Ok(output) => {
                    if output.status.code() != Some(0) {
                        return Err(format!(
                            "Failed to run `pdf2svg`: {}",
                            String::from_utf8_lossy(&output.stderr)
                        ));
                    }
                }
                Err(err) => {
                    return Err(format!("Failed to run `pdf2svg`: {err:?}"));
                }
            }
        }

        // Delete temporary directory
        fs::remove_dir_all(TMP_DIRECTORY).unwrap();

        Ok(svg_filename)
    }
}
