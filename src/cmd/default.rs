//! Default capsa management command module

use std::io;
use std::fs;

pub fn run(
    ctx: &emx_note::ResolveContext,
    caps: Option<String>,
) -> io::Result<()> {
    if let Some(caps_name) = caps {
        // Set default: write to .default file
        let _capsa_ref = super::resolve::resolve_capsa(ctx, Some(&caps_name))?;
        let default_file = ctx.home.join(".default");

        fs::write(&default_file, &caps_name)?;
        eprintln!("Default capsa set to '{}'", caps_name);
        Ok(())
    } else {
        // Get default: print current default path
        let capsa_ref = super::resolve::resolve_capsa(ctx, None)?;
        println!("{}", emx_note::util::display_path(&capsa_ref.path));
        Ok(())
    }
}
