#[macro_use]
extern crate clap;

use ffphotojoin::image::imageops::FilterType;
use ffphotojoin::image::io::Reader;
use ffphotojoin::image::{DynamicImage, GenericImageView};
use ffphotojoin::{Direction, Sizing};
use std::path::PathBuf;

const DEFAULT_SIZING: Sizing = Sizing::ToSmallest;

fn main() {
    // Create argument parser
    let arg_matcher = clap_app!(ffphotojoin_cli =>
        (version: std::env!("CARGO_PKG_VERSION"))
        (author: std::env!("CARGO_PKG_AUTHORS"))
        (about: std::env!("CARGO_PKG_DESCRIPTION"))
        (@arg input: -i --input +multiple +required +takes_value "Provides an input image or images to the joiner")
        (@arg output: -o --output +required +takes_value "Set the image output file (PNG or JPEG formats only)")
        (@arg direction: -d --direction +required +takes_value "Set the direction of the output image (vertical/horizontal)")
        (@arg filter: --filter +takes_value "Set the filter to use when resizing images (nearest/triangle/catmull_rom/gaussian/lanczos3)")
        (@arg override_output: -f --override_output "Overrides the output file if it exists when present")
        (@arg size_to_largest: -l --size_to_largest "Resize all images (keeping the aspect ratio) to fit the size of the largest image")
        (@arg size_to_smallest: -s --size_to_smallest "Resize all images (keeping the aspect ratio) to fit the size of the smallest image")
    ).get_matches();

    // Load arguments from parser
    let inputs = arg_matcher
        .values_of("input")
        .expect("no input files/directories provided")
        .into_iter()
        .map(|input| PathBuf::from(shellexpand::tilde(input).as_ref()))
        .collect::<Vec<_>>();
    let output_path = PathBuf::from(
        shellexpand::tilde(arg_matcher.value_of("output").expect("no output file")).as_ref(),
    );
    let direction = {
        let d = arg_matcher
            .value_of("direction")
            .expect("no direction provided")
            .to_lowercase();
        match d.as_str() {
            "vertical" => Direction::Vertical,
            _ => Direction::Horizontal,
        }
    };
    let filter = {
        if let Some(filter) = arg_matcher.value_of("filter") {
            match filter.to_lowercase().as_str() {
                "nearest" => FilterType::Nearest,
                "triangle" => FilterType::Triangle,
                "catmull_rom" => FilterType::CatmullRom,
                "gaussian" => FilterType::Gaussian,
                "lanczos3" => FilterType::Lanczos3,
                _ => FilterType::Gaussian,
            }
        } else {
            FilterType::Gaussian
        }
    };
    let override_output = arg_matcher.is_present("override_output");
    let size_to_largest = arg_matcher.is_present("size_to_largest");
    let size_to_smallest = arg_matcher.is_present("size_to_smallest");

    println!(
        "Joining photos {} with filter: {:?}",
        match direction {
            Direction::Horizontal => "horizontally",
            Direction::Vertical => "vertically",
        },
        filter
    );

    // Determine how to size the output image
    if size_to_largest && size_to_smallest {
        panic!("only one size argument may be provided");
    }
    let sizing = if size_to_smallest {
        Sizing::ToSmallest
    } else if size_to_largest {
        Sizing::ToLargest
    } else {
        DEFAULT_SIZING
    };
    match sizing {
        Sizing::ToSmallest => println!("Resizing to smallest image"),
        Sizing::ToLargest => println!("Resizing to largest image"),
    }

    // Join the photos
    let output_image = ffphotojoin::join_photos(
        load_images(inputs),
        ffphotojoin::PhotoJoinOptions {
            direction,
            sizing,
            filter,
        },
    )
    .expect("failed to join photos");

    // Write the output image
    if output_path.exists() && !override_output {
        panic!("output file already exists");
    }
    println!(
        "Generated {}x{} image",
        output_image.width(),
        output_image.height(),
    );
    output_image
        .save(&output_path)
        .expect("failed to save image to output file");
    println!("Saved joined photo to {}", output_path.to_str().unwrap());
}

fn load_images(files: Vec<PathBuf>) -> Vec<DynamicImage> {
    files
        .into_iter()
        .map(|file| {
            println!("Opening {}", file.to_str().unwrap());
            Reader::open(file)
                .expect("failed to open image file")
                .decode()
                .expect("failed to decode image")
        })
        .collect()
}
