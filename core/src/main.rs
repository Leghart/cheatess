mod utils;

fn main() {
    utils::runner();
    // let screen_area = utils::get_screen_area().unwrap();
    // let start = std::time::Instant::now();
    // let image = utils::take_screenshot(&screen_area).unwrap();
    // // image.save(std::path::Path::new("org.png"));
    // let new_image = utils::to_binary(&image, 70);
    // // new_image.save(std::path::Path::new("dupa.png"));
    // let trimmed = utils::trimm(&new_image);
    // trimmed.save(std::path::Path::new("dupa.png"));

    // utils::create_piece_templates(&trimmed, utils::Color::BLACK);
    // println!("{:?}", start.elapsed());
    // trimmed.save(std::path::Path::new("trimmed.png"));
}
