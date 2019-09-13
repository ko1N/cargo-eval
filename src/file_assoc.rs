//! This module deals with setting up file associations.
//! Since this only makes sense on Windows, this entire module is Windows-only.

use std::io;

use clap;
use itertools::Itertools;
use winreg::{enums as wre, RegKey};

use crate::error::{Blame, Result};

#[derive(Debug)]
pub enum Args {
    Install { amend_pathext: bool },
    Uninstall,
}

impl Args {
    pub fn subcommand() -> clap::App<'static, 'static> {
        use clap::{AppSettings, Arg, SubCommand};

        SubCommand::with_name("file-association")
            .about("Manage file assocations.")
            .setting(AppSettings::SubcommandRequiredElseHelp)
            .subcommand(SubCommand::with_name("install")
                .about("Install file associations.")
                .arg(Arg::with_name("amend_pathext")
                    .help("Add script extension to PATHEXT.  This allows scripts to be executed without typing the file extension.")
                    .long("amend-pathext")
                )
            )
            .subcommand(SubCommand::with_name("uninstall")
                .about("Uninstall file associations.")
            )
    }

    pub fn parse(m: &clap::ArgMatches) -> Self {
        match m.subcommand() {
            ("install", Some(m)) => Args::Install {
                amend_pathext: m.is_present("amend_pathext"),
            },
            ("uninstall", _) => Args::Uninstall,
            (name, _) => panic!("bad subcommand: {:?}", name),
        }
    }
}

pub fn try_main(args: Args) -> Result<i32> {
    match args {
        Args::Install { amend_pathext } => install(amend_pathext)?,
        Args::Uninstall => uninstall()?,
    }

    Ok(0)
}

fn install(amend_pathext: bool) -> Result<()> {
    use std::env;

    // Set up file association.
    let cargo_eval_path = env::current_exe()?;
    let cargo_eval_path = cargo_eval_path.canonicalize()?;

    // We have to remove the `\\?\` prefix because, if we don't, the shell freaks out.
    let cargo_eval_path = cargo_eval_path.to_string_lossy();
    let cargo_eval_path = if cargo_eval_path.starts_with(r#"\\?\"#) {
        &cargo_eval_path[4..]
    } else {
        &cargo_eval_path[..]
    };

    let res = (|| -> io::Result<()> {
        let hlcr = RegKey::predef(wre::HKEY_CLASSES_ROOT);
        let (dot_crs, _) = hlcr.create_subkey(".crs")?;
        dot_crs.set_value("", &"CargoScript.Crs")?;

        let (cargo_eval_crs, _) = hlcr.create_subkey("CargoScript.Crs")?;
        cargo_eval_crs.set_value("", &"Cargo Script")?;

        let (sh_o_c, _) = cargo_eval_crs.create_subkey(r#"shell\open\command"#)?;
        sh_o_c.set_value("", &format!(r#""{}" "--" "%1" %*"#, cargo_eval_path))?;
        Ok(())
    })();

    match res {
        Ok(()) => (),
        Err(e) => {
            if e.kind() == io::ErrorKind::PermissionDenied {
                println!(
                    "Access denied.  Make sure you run this command from an administrator prompt."
                );
                return Err((Blame::Human, e).into());
            } else {
                return Err(e.into());
            }
        }
    }

    println!("Created cargo-eval registry entry.");
    println!("- Handler set to: {}", cargo_eval_path);

    // Amend PATHEXT.
    if amend_pathext {
        let hklm = RegKey::predef(wre::HKEY_LOCAL_MACHINE);
        let env =
            hklm.open_subkey(r#"SYSTEM\CurrentControlSet\Control\Session Manager\Environment"#)?;

        let pathext: String = env.get_value("PATHEXT")?;
        if !pathext.split(';').any(|e| e.eq_ignore_ascii_case(".crs")) {
            let pathext = pathext.split(';').chain(Some(".CRS")).join(";");
            env.set_value("PATHEXT", &pathext)?;
        }

        println!(
            "Added `.crs` to PATHEXT.  You may need to log out for the change to take effect."
        );
    }

    Ok(())
}

fn uninstall() -> Result<()> {
    let hlcr = RegKey::predef(wre::HKEY_CLASSES_ROOT);
    hlcr.delete_subkey(r#"CargoScript.Crs\shell\open\command"#)
        .ignore_missing()?;
    hlcr.delete_subkey(r#"CargoScript.Crs\shell\open"#)
        .ignore_missing()?;
    hlcr.delete_subkey(r#"CargoScript.Crs\shell"#)
        .ignore_missing()?;
    hlcr.delete_subkey(r#"CargoScript.Crs"#).ignore_missing()?;

    println!("Deleted cargo-eval registry entry.");

    {
        let hklm = RegKey::predef(wre::HKEY_LOCAL_MACHINE);
        let env =
            hklm.open_subkey(r#"SYSTEM\CurrentControlSet\Control\Session Manager\Environment"#)?;

        let pathext: String = env.get_value("PATHEXT")?;
        if pathext.split(';').any(|e| e.eq_ignore_ascii_case(".crs")) {
            let pathext = pathext
                .split(';')
                .filter(|e| !e.eq_ignore_ascii_case(".crs"))
                .join(";");
            env.set_value("PATHEXT", &pathext)?;
            println!("Removed `.crs` from PATHEXT.  You may need to log out for the change to take effect.");
        }
    }

    Ok(())
}

trait IgnoreMissing {
    fn ignore_missing(self) -> Self;
}

impl IgnoreMissing for io::Result<()> {
    fn ignore_missing(self) -> Self {
        if let Err(ref e) = self {
            if e.kind() == io::ErrorKind::NotFound {
                return Ok(());
            }
        }

        self
    }
}
