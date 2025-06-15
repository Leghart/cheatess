use image::{ImageBuffer, Rgba};
use opencv::{
    core::{split, Mat, Point, Rect, Scalar, Size, Vector, CV_8UC4},
    highgui::{self, destroy_window},
    imgcodecs, imgproc,
    prelude::*,
};

pub struct ImageProcessing {}

impl ImageProcessing {
    pub fn read(path: &str) -> Result<Mat, Box<dyn std::error::Error>> {
        Ok(imgcodecs::imread(path, imgcodecs::IMREAD_UNCHANGED)?)
    }
    pub fn write(path: &str, image: &Mat) -> Result<bool, Box<dyn std::error::Error>> {
        let params = Vector::from_iter([0, 16]);
        Ok(imgcodecs::imwrite(path, &image, &params)?)
    }

    pub fn show(image: &Mat, destroy: bool) -> Result<(), Box<dyn std::error::Error>> {
        highgui::imshow("test_window", &image)?;
        loop {
            if highgui::wait_key(0)? == 48 {
                break;
            }
        }
        if destroy {
            destroy_window("test_window")?;
        }
        Ok(())
    }

    pub fn image_buffer_to_mat(
        buffer: ImageBuffer<Rgba<u8>, Vec<u8>>,
    ) -> Result<Mat, Box<dyn std::error::Error>> {
        let (width, height) = buffer.dimensions();
        let mut mat = Mat::new_rows_cols_with_default(
            height as i32,
            width as i32,
            CV_8UC4,
            Scalar::all(0.0),
        )?;

        // let mat_data = mat.data_bytes_mut()?;
        // mat_data.copy_from_slice(&buffer.as_raw());

        let mut mat_bgr = Mat::default();
        imgproc::cvt_color(&mat, &mut mat_bgr, imgproc::COLOR_RGBA2BGR, 0)?;

        Ok(mat_bgr)
    }

    pub fn draw_box(
        image: &mut Mat,
        start_point: &Point,
        size: &Size,
        color: Scalar,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let rect = Rect::new(start_point.x, start_point.y, size.width, size.height);
        imgproc::rectangle(image, rect, color, 2, 8, 0)?;
        Ok(())
    }

    pub fn resize(image: &Mat, width: i32, height: i32) -> Result<Mat, Box<dyn std::error::Error>> {
        let nsize = Size::new(width, height);

        let mut resized = Mat::default();
        imgproc::resize(&image, &mut resized, nsize, 0.0, 0.0, imgproc::INTER_LINEAR)?;

        Ok(resized)
    }

    pub fn threshold(image: &Mat) -> Result<Mat, Box<dyn std::error::Error>> {
        let mut thresholded = Mat::default();
        imgproc::cvt_color(image, &mut thresholded, imgproc::COLOR_BGR2GRAY, 0)?;

        Ok(thresholded)
    }

    pub fn match_template(
        image_bg: &Mat,
        image_fg: &Mat,
        mask: &Mat,
    ) -> Result<Mat, Box<dyn std::error::Error>> {
        let mut matched = Mat::default();
        imgproc::match_template(
            image_bg,
            image_fg,
            &mut matched,
            imgproc::TM_SQDIFF_NORMED,
            &mask,
        )?;
        Ok(matched)
    }

    pub fn get_mask(image: &Mat) -> Result<Mat, Box<dyn std::error::Error>> {
        let mut channels: Vector<Mat> = Vector::new();
        split(image, &mut channels)?;
        let alpha = channels.get(3)?;

        let mut binary_mask = Mat::default();
        opencv::imgproc::threshold(
            &alpha,
            &mut binary_mask,
            1.0,
            255.0,
            opencv::imgproc::THRESH_BINARY,
        )?;

        Ok(binary_mask)
    }
}
