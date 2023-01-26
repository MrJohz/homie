use sqlx::migrate::Migrator;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

fn main() {
    println!("Hello, World!")
}
