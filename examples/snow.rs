mod utils;

use std::error::Error;

use tui_rain::Rain;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    utils::render_rain(Box::new(|elapsed| Rain::new_snow(elapsed))).await
}
