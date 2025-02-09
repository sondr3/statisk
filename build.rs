use std::{env, fs::File, io::Error, path::Path};

use clap::{CommandFactory, ValueEnum};
use clap_complete::generate_to;
use clap_mangen::Man;

include!("src/cli.rs");

fn build_shell_completion(outdir: &Path) -> Result<(), Error> {
    let mut app = Options::command();
    let shells = Shell::value_variants();

    for shell in shells {
        generate_to(*shell, &mut app, "statisk", outdir)?;
    }

    Ok(())
}

fn build_manpages(outdir: &Path) -> Result<(), Error> {
    let app = Options::command();

    let file = outdir.join("statisk.1");
    let mut file = File::create(file)?;

    Man::new(app).render(&mut file)?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = match env::var_os("OUT_DIR") {
        None => return Ok(()),
        Some(out_dir) => out_dir,
    };

    let out_path = PathBuf::from(out_dir);
    let mut path = out_path.ancestors().nth(4).unwrap().to_owned();
    path.push("assets");
    std::fs::create_dir_all(&path).unwrap();

    build_shell_completion(&path)?;
    build_manpages(&path)?;

    Ok(())
}
