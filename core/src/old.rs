mod utils;

extern crate image;
extern crate opencv;

use opencv::{
    core::{Mat, Vector, CV_8UC1},
    highgui, imgcodecs, imgproc,
    objdetect::CascadeClassifier,
    prelude::*,
};

fn main() {
    // utils::runner();
    let screen_area = utils::get_screen_area().unwrap();
    // let start: std::time::Instant = std::time::Instant::now();
    let image = utils::take_screenshot(&screen_area).unwrap();
    let mat = utils::to_mat(&image);
    let binary = utils::to_binary(&mat);
    let resized = utils::resize(&binary, 200, 200).unwrap();

    // let trimmed = utils::trimm(&new_image);
    // println!("{:?}", start.elapsed());
    utils::save_mat(&resized, "before.png");
    // resized.save(std::path::Path::new("dupa.png"));

    // let img = imgcodecs::imread("dupa.png", imgcodecs::IMREAD_COLOR)
    //     .expect("Nie udało się wczytać obrazu");

    // // Konwersja obrazu do skali szarości
    // let mut gray = Mat::default();
    // imgproc::cvt_color(&img, &mut gray, imgproc::COLOR_BGR2GRAY, 0).unwrap();

    // // Wykrywanie krawędzi
    // let mut edges = Mat::default();
    // imgproc::canny(&gray, &mut edges, 100.0, 200.0, 3, false).unwrap();

    // let mut params = Vector::new();
    // params.push(16); // Parametr ID dla kompresji PNG.
    // params.push(9); // Stopień kompresji (9 = najwyższa jakość).

    // // Zapisanie przetworzonego obrazu
    // imgcodecs::imwrite("edges.png", &edges, &params).unwrap();

    // // Wyświetlanie wyników
    // highgui::imshow("Edges", &edges).unwrap();
    // highgui::wait_key(0).unwrap();

    // let image1 = imgcodecs::imread("before.png", imgcodecs::IMREAD_COLOR).unwrap();
    // let image2 = imgcodecs::imread("after.png", imgcodecs::IMREAD_COLOR).unwrap();

    // // Detekcja konturów na obu obrazach
    // let contours1 = utils::detect_contours(&image1).unwrap();
    // let contours2 = utils::detect_contours(&image2).unwrap();
    // println!("{:?}", contours1);
    // println!("{:?}", contours2);
    // // Porównanie konturów
    // utils::compare_contours(&contours1, &contours2);
}
