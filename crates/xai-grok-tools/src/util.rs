/// Detach a command from the terminal by setting up a new process group.
pub fn detach_std_command(cmd: &mut std::process::Command) {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        unsafe {
            cmd.pre_exec(|| {
                // Create a new process group to detach from terminal.
                libc::setsid();
                Ok(())
            });
        }
    }
    let _ = cmd;
}

/// Image validation utilities.
pub mod image_validate {
    use image::GenericImageView;

    /// Validate image bytes unrestricted — accepts any format that the
    /// `image` crate recognises. Returns `(width, height, color_type)`.
    pub fn validate_image_bytes_unrestricted(
        bytes: &[u8],
        _check_alpha: bool,
    ) -> Result<(u32, u32, String), String> {
        match image::load_from_memory(bytes) {
            Ok(img) => {
                let (w, h) = img.dimensions();
                let color = format!("{:?}", img.color());
                Ok((w, h, color))
            }
            Err(e) => Err(format!("image validation failed: {e}")),
        }
    }
}
