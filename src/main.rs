use std::env;
use std::thread;
use std::sync::Arc;
use utils::{create_output_directory, process_video, extract_audio}; // Import the `extract_audio` function

mod utils;

pub fn convert_video() -> Result<(), ffmpeg_next::Error> {
    // Get the input file from command line argument
    let input_file = env::args().nth(1).expect("Cannot open file.");
    let interval = env::args().nth(2).unwrap_or_else(|| "30".to_string()).parse::<usize>().unwrap(); // Default to every 30 frames

    let output_dir = "output";
    
    // Remove the directory if it exists
    if std::path::Path::new(output_dir).exists() {
        std::fs::remove_dir_all(output_dir).unwrap();
    }
    create_output_directory(output_dir).unwrap();

    // Clone input_file and output_dir to move into threads
    let input_file_audio = Arc::new(input_file.clone());
    let output_dir_audio = Arc::new(output_dir.to_string());

    // Run video processing in a separate thread
    let video_thread = thread::spawn(move || {
        match process_video(&input_file, output_dir, interval) {
            Ok(_) => println!("Video processing completed successfully."),
            Err(e) => eprintln!("Error processing video: {}", e),
        }
    });

    // Run audio extraction concurrently
    let audio_thread = thread::spawn(move || {
        let output_audio_file = format!("{}/audio.mp3", output_dir_audio); // Use the output directory for `audio.mp3`
        match extract_audio(&input_file_audio, &output_audio_file) {
            Ok(_) => println!("Audio extraction completed successfully."),
            Err(e) => eprintln!("Error extracting audio: {}", e),
        }
    });

    // Wait for both threads to complete
    video_thread.join().unwrap();
    audio_thread.join().unwrap();

    Ok(())
}

fn main() {
    convert_video().unwrap();
}
