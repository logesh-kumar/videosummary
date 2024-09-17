extern crate ffmpeg_next as ffmpeg;

use ffmpeg::format::{input, Pixel};
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{context::Context, flag::Flags};
use ffmpeg::util::frame::video::Video;
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::Path;

// Function to process the video and extract frames
pub fn process_video(input_file: &str, output_dir: &str, interval: usize) -> Result<(), ffmpeg::Error> {
    ffmpeg::init().unwrap();

    // Open the input video file
    let mut ictx = input(input_file)?;

    // Get the best video stream
    let input = ictx
        .streams()
        .best(Type::Video)
        .ok_or(ffmpeg::Error::StreamNotFound)?;
    let video_stream_index = input.index();

    // Initialize the decoder and scaler
    let mut decoder = initialize_decoder(&input)?;
    let mut scaler = initialize_scaler(&decoder)?;

    // Process the packets and extract frames
    extract_frames(&mut ictx, &mut decoder, &mut scaler, video_stream_index, output_dir, interval)?;

    Ok(())
}

// Function to initialize the video decoder
fn initialize_decoder(input: &ffmpeg::Stream) -> Result<ffmpeg::decoder::Video, ffmpeg::Error> {
    let context_decoder = ffmpeg::codec::context::Context::from_parameters(input.parameters())?;
    Ok(context_decoder.decoder().video()?)
}

// Function to initialize the scaler
fn initialize_scaler(decoder: &ffmpeg::decoder::Video) -> Result<Context, ffmpeg::Error> {
    Context::get(
        decoder.format(),
        decoder.width(),
        decoder.height(),
        Pixel::RGB24,
        decoder.width(),
        decoder.height(),
        Flags::BILINEAR,
    )
}

// Function to extract frames from the video at the specified interval
fn extract_frames(
    ictx: &mut ffmpeg::format::context::Input,
    decoder: &mut ffmpeg::decoder::Video,
    scaler: &mut Context,
    video_stream_index: usize,
    output_dir: &str, // output directory parameter
    interval: usize,  // extraction interval
) -> Result<(), ffmpeg::Error> {
    let mut frame_index = 0;
    let mut extracted_frame_index = 0;

    // Closure to process and save decoded frames
    let mut process_decoded_frames = |decoder: &mut ffmpeg::decoder::Video| -> Result<(), ffmpeg::Error> {
        let mut decoded = Video::empty();
        while decoder.receive_frame(&mut decoded).is_ok() {
            // Save frame if it's on the specified interval
            if frame_index % interval == 0 {
                let mut rgb_frame = Video::empty();
                scaler.run(&decoded, &mut rgb_frame)?;
                save_frame(&rgb_frame, extracted_frame_index, output_dir).unwrap();
                extracted_frame_index += 1;
            }
            frame_index += 1;
        }
        Ok(())
    };

    // Iterate over packets in the input context
    for (stream, packet) in ictx.packets() {
        if stream.index() == video_stream_index {
            decoder.send_packet(&packet)?;
            process_decoded_frames(decoder)?;
        }
    }

    // Handle the end of the stream
    decoder.send_eof()?;
    process_decoded_frames(decoder)?;

    Ok(())
}

// Function to save the frame as a PPM file in the output directory
fn save_frame(frame: &Video, index: usize, output_dir: &str) -> std::result::Result<(), std::io::Error> {
    let file_path = format!("{}/frame{}.png", output_dir, index);
    let mut file = File::create(file_path)?;
    file.write_all(format!("P6\n{} {}\n255\n", frame.width(), frame.height()).as_bytes())?;
    file.write_all(frame.data(0))?;
    Ok(())
}

// Helper function to create the output directory if it doesn't exist
pub fn create_output_directory(output_dir: &str) -> std::result::Result<(), std::io::Error> {
    if !Path::new(output_dir).exists() {
        fs::create_dir(output_dir)?;
    }
    Ok(())
}


