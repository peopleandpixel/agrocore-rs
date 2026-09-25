fn main() {
    let api_base = std::env::var("AGROCORE_API_BASE_URL")
        .unwrap_or_else(|_| "http://localhost:8081".to_string());
    println!("cargo:rustc-env=AGROCORE_API_BASE_URL={}", api_base);
}
