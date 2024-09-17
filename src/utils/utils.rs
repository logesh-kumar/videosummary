extern crate ffmpeg_next as ffmpeg;

use ffmpeg::format::{input, Pixel};
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{context::Context, flag::Flags};
use ffmpeg::util::frame::video::Video;
use std::fs::{self};
use std::path::Path;
use rayon::prelude::*;
use image::{ImageBuffer, Rgb};

pub fn process_video(input_file: &str, output_dir: &str, interval: usize) -> Result<(), ffmpeg::Error> {
    ffmpeg::init().unwrap();

    create_output_directory(output_dir).unwrap();

    extract_frames_from_video(input_file, output_dir, interval)?;

    Ok(())
}

/**
 * Detailed explanation of the `extract_frames_from_video` function:
 * This function extracts frames from a video file at a specified interval and saves them as PNG images.
 * How it works:
 * 1. Initialize the input context from the video file.
 * 2. Get the video stream from the input context.
 * 3. Initialize the decoder and scaler for video processing.
 * 4. Iterate over packets in the input context.
 * 5. Send the packet to the decoder and receive decoded frames.
 * 6. Scale the decoded frames to RGB format.
 * 7. Save the frames at the specified interval using Rayon for parallel processing.
 * 8. Return the result.
 */
fn extract_frames_from_video(input_file: &str, output_dir: &str, interval: usize) -> Result<(), ffmpeg::Error> {
    let mut ictx = input(input_file)?;

    let input = ictx
        .streams()
        .best(Type::Video)
        .ok_or(ffmpeg::Error::StreamNotFound)?;
    let video_stream_index = input.index();

    let mut decoder = initialize_decoder(&input)?;
    let mut scaler = initialize_scaler(&decoder)?;

    let mut frame_index = 0;
    let mut extracted_frames = Vec::new();

    for (stream, packet) in ictx.packets() {
        if stream.index() == video_stream_index {
            decoder.send_packet(&packet)?;
            
            let mut decoded = Video::empty();
            while decoder.receive_frame(&mut decoded).is_ok() {
                if frame_index % interval == 0 {
                    let mut rgb_frame = Video::empty();
                    scaler.run(&decoded, &mut rgb_frame)?;
                    extracted_frames.push((frame_index / interval, rgb_frame));
                }
                frame_index += 1;
            }
        }
    }

    decoder.send_eof()?;
    let mut decoded = Video::empty();
    while decoder.receive_frame(&mut decoded).is_ok() {
        if frame_index % interval == 0 {
            let mut rgb_frame = Video::empty();
            scaler.run(&decoded, &mut rgb_frame)?;
            extracted_frames.push((frame_index / interval, rgb_frame));
        }
        frame_index += 1;
    }

    // Use Rayon for parallel processing of frame saving
    extracted_frames.par_iter().for_each(|(index, frame)| {
        if let Err(e) = save_frame_optimized(frame, *index, output_dir) {
            eprintln!("Error saving frame {}: {}", index, e);
        }
    });

    Ok(())
}

fn initialize_decoder(input: &ffmpeg::Stream) -> Result<ffmpeg::decoder::Video, ffmpeg::Error> {
    let context_decoder = ffmpeg::codec::context::Context::from_parameters(input.parameters())?;
    context_decoder.decoder().video()
}

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

fn save_frame_optimized(frame: &Video, index: usize, output_dir: &str) -> std::io::Result<()> {
    let file_path = format!("{}/frame{}.png", output_dir, index);
    
    // Convert raw bytes to image and encode as PNG
    let img = ImageBuffer::<Rgb<u8>, _>::from_raw(
        frame.width(),
        frame.height(),
        frame.data(0).to_vec()
    ).unwrap();
    
    img.save(file_path).unwrap();

    Ok(())
}

pub fn create_output_directory(output_dir: &str) -> std::io::Result<()> {
    if !Path::new(output_dir).exists() {
        fs::create_dir(output_dir)?;
    }
    Ok(())
}