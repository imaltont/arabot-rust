mod config;

fn main() {
    println!("Hello, world! Name: {}, Oauth: {}", &config::CONFIG.name, &config::CONFIG.oauth);
}
