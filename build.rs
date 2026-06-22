use std::io;

fn main() -> io::Result<()> {
    #[cfg(windows)]
    {
        winresource::WindowsResource::new()
            .set_icon("assets/spaceman.ico")
            .compile()?;
    }
    Ok(())
}
