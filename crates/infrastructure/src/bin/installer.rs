use std::fs;
use std::path::Path;
use std::io::{self, Write};

fn main() -> io::Result<()> {
    println!("=== AgroCore Systemd Installer ===");

    let services = vec![
        ("agrocore-api", "AgroCore REST API & Admin UI Service", "/usr/local/bin/agrocore-api"),
        ("agrocore-weather", "AgroCore Weather Service", "/usr/local/bin/agrocore-weather"),
        ("agrocore-reporting", "AgroCore Reporting Service", "/usr/local/bin/agrocore-reporting"),
    ];

    let user = "agrocore";
    let group = "agrocore";
    let config_dir = "/etc/agrocore";
    let data_dir = "/var/lib/agrocore";

    println!("\nVorbereitung der Konfigurationsverzeichnisse...");
    if !Path::new(config_dir).exists() {
        println!("Erstelle {}...", config_dir);
        // In einer echten Umgebung bräuchten wir hier sudo/root Rechte
        // Für diesen Installer generieren wir die Files lokal in einem 'systemd' Ordner
    }

    let output_dir = "systemd_units";
    fs::create_dir_all(output_dir)?;

    for (name, description, exec) in services {
        let unit_content = format!(
r#"[Unit]
Description={}
After=network.target mongodb.service nats-server.service

[Service]
Type=simple
User={}
Group={}
WorkingDirectory={}
ExecStart={}
Restart=always
RestartSec=5
Environment=RUST_LOG=info
EnvironmentFile=-{}/.env

[Install]
WantedBy=multi-user.target
"#, description, user, group, config_dir, exec, config_dir);

        let file_path = format!("{}/{}.service", output_dir, name);
        let mut file = fs::File::create(&file_path)?;
        file.write_all(unit_content.as_bytes())?;
        println!("Generiert: {}", file_path);
    }

    println!("\nNächste Schritte:");
    println!("1. Benutzer anlegen: sudo useradd -r -m -d {} -s /bin/false {}", data_dir, user);
    println!("2. Binaries nach /usr/local/bin/ kopieren.");
    println!("3. Admin-UI Dateien nach {}/admin-ui kopieren.", data_dir);
    println!("4. Unit-Files nach /etc/systemd/system/ kopieren.");
    println!("5. Services starten: sudo systemctl enable --now agrocore-api");
    println!("\nDie Admin-Oberfläche wird unter http://<server-ip>:3000/admin erreichbar sein.");

    Ok(())
}
