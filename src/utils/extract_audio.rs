use std::process::Command;
use std::path::Path;

pub fn extract_audio(input_file: &str, output_file: &str) -> Result<(), std::io::Error> {
    // Ensure the input file exists
    if !Path::new(input_file).exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Input file not found",
        ));
    }

    // Run the ffmpeg command to extract audio
    let output = Command::new("ffmpeg")
        .args(&["-i", input_file, "-b:a", "192K", "-vn", output_file]) // '-b:a' sets audio bitrate, '-vn' disables video
        .output()?;

    // Handle output and error
    if output.status.success() {
        println!("Audio extraction completed successfully!");
        println!("Standard Output: {}", String::from_utf8_lossy(&output.stdout));
    } else {
        println!("Audio extraction failed!");
        println!("Standard Error: {}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(())
}
