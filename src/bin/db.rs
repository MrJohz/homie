use clap::Parser;

#[derive(clap::Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Migrates a database to the latest version
    Migrate,
    /// Adds a new user to the database
    AddUser {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        password: String,
    },
    /// Adds a new task to the database
    AddTask,
}

#[tokio::main]
async fn main() {
    let cli = Args::parse();
    match &cli.command {
        Commands::Migrate => {
            let conn = homie::db::create_connection().await;
            homie::db::migrate(conn).await.unwrap();
        }
        Commands::AddUser { name, password } => {
            let conn = homie::db::create_connection().await;
            let store = homie::auth::AuthStore::new(conn);
            store.create_user(name, password).await.unwrap();
        }
        Commands::AddTask => {
            dbg!("Adding task");
        }
    }
}
