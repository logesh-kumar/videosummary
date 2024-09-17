use std::env;

use utils::{create_output_directory, process_video, extract_audio}; // Import the `extract_audio` function

mod utils;

pub fn convert_video() -> Result<(), ffmpeg_next::Error> {
    // Get the input file from command line argument
    let input_file = env::args().nth(1).expect("Cannot open file.");

    // Define frame extraction interval (every N frames)
    let interval = env::args().nth(2).unwrap_or_else(|| "30".to_string()).parse::<usize>().unwrap(); // Default to every 30 frames

    // Create output_frames directory if it doesn't exist
    let output_dir = "output";
    create_output_directory(output_dir).unwrap();

    // Process the video and extract frames
    match process_video(&input_file, output_dir, interval) {
        Ok(_) => {
            println!("Video processing completed successfully.");

            println!("Extracting audio from the video {}", input_file);
            
            // Define the output audio file using the output_dir and input_file            
            let output_audio_file = format!("{}/audio.mp3", output_dir); // Use the output directory with `audio.mp3` file name
            
            // Call the `extract_audio` function to extract audio from the video
            match extract_audio(&input_file, &output_audio_file) {
                Ok(_) => println!("Audio extraction completed successfully."),
                Err(e) => eprintln!("Error extracting audio: {}", e),
            }
        }
        Err(e) => eprintln!("Error processing video: {}", e),
    }

    Ok(())
}

fn main() {
    convert_video().unwrap();
}
