pub use image;
use image::imageops::FilterType;
use image::{imageops, DynamicImage, GenericImageView};

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum Direction {
    Horizontal,
    Vertical,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum Sizing {
    ToSmallest,
    ToLargest,
}

#[derive(Copy, Clone)]
pub struct PhotoJoinOptions {
    pub direction: Direction,
    pub sizing: Sizing,
    pub filter: FilterType,
}

#[derive(Copy, Clone, Debug)]
pub struct NoImagesProvided;

pub fn join_photos(
    photos: Vec<DynamicImage>,
    options: PhotoJoinOptions,
) -> Result<DynamicImage, NoImagesProvided> {
    // Just leave if the images are empty (shouldn't happen basically)
    if photos.is_empty() {
        return Err(NoImagesProvided);
    }
    if photos.len() == 1 {
        return Ok(photos.into_iter().next().unwrap());
    }
    println!("Joining {} photos", photos.len());

    // Determine the sizes for the output image
    let perpendicular_size = photos.iter().fold(
        match options.sizing {
            Sizing::ToSmallest => u32::MAX,
            Sizing::ToLargest => 0,
        },
        |size, img| {
            let dir_size = match options.direction {
                Direction::Horizontal => img.width(),
                Direction::Vertical => img.height(),
            };
            match options.sizing {
                Sizing::ToSmallest => size.min(dir_size),
                Sizing::ToLargest => size.max(dir_size),
            }
        },
    );
    let join_size = photos.iter().fold(0u32, |size, img| {
        let scale = get_scale_factor(perpendicular_size, options.direction, &img);
        size + (scale
            * match options.direction {
                Direction::Horizontal => img.width(),
                Direction::Vertical => img.height(),
            } as f32) as u32
    });
    println!(
        "Determined output image size: {}",
        match options.direction {
            Direction::Horizontal => format!("{}x{}", perpendicular_size, join_size),
            Direction::Vertical => format!("{}x{}", join_size, perpendicular_size),
        }
    );

    // Resize the first image to the full size of the output
    // We should be able to use `photos.first().unwrap()` safely because we know there is at least
    //  1 image provided
    let mut output_img = photos.first().unwrap().resize_exact(
        match options.direction {
            Direction::Horizontal => join_size,
            Direction::Vertical => perpendicular_size,
        },
        match options.direction {
            Direction::Horizontal => perpendicular_size,
            Direction::Vertical => join_size,
        },
        FilterType::Nearest,
    );

    let _ = photos.into_iter().fold(0u32, |pos, img| {
        // Get sizing and positioning information
        let (w, h) = get_size(perpendicular_size, options.direction, &img);
        let (x, y) = match options.direction {
            Direction::Horizontal => (pos, 0),
            Direction::Vertical => (0, pos),
        };

        // Overlay the resized image on top of the final image
        imageops::overlay(
            &mut output_img,
            &imageops::resize(&img, w, h, options.filter),
            x,
            y,
        );
        println!("Overlayed image at {},{} with size {}x{}", x, y, w, h);

        // Accumulate size in the join direction
        match options.direction {
            Direction::Vertical => pos + h,
            Direction::Horizontal => pos + w,
        }
    });

    Ok(output_img)
}

fn get_scale_factor(perpendicular_size: u32, direction: Direction, img: &DynamicImage) -> f32 {
    perpendicular_size as f32
        / match direction {
            Direction::Horizontal => img.height(),
            Direction::Vertical => img.width(),
        } as f32
}

fn get_size(perpendicular_size: u32, direction: Direction, img: &DynamicImage) -> (u32, u32) {
    let scale_factor = get_scale_factor(perpendicular_size, direction, img);
    match direction {
        Direction::Horizontal => (
            (scale_factor * img.width() as f32) as u32,
            perpendicular_size,
        ),
        Direction::Vertical => (
            perpendicular_size,
            (scale_factor * img.height() as f32) as u32,
        ),
    }
}
