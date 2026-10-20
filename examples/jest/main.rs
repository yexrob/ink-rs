// Ink examples/jest.

mod app;

use ink_rs::Options;

#[tokio::main(flavor = "local")]
async fn main() -> Result<(), ink_rs::Error> {
    ink_rs::tokio::render(app::app(), Options::default())
        .wait_until_exit()
        .await
        .map(|_| ())
}
