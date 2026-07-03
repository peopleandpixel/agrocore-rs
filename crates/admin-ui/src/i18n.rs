use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    EN,
    DE,
    ES,
    FR,
    PT,
}

impl Language {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "de" => Language::DE,
            "es" => Language::ES,
            "fr" => Language::FR,
            "pt" => Language::PT,
            _ => Language::EN,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Language::EN => "en",
            Language::DE => "de",
            Language::ES => "es",
            Language::FR => "fr",
            Language::PT => "pt",
        }
    }
}

#[derive(Clone, Default)]
pub struct I18n {
    translations: HashMap<&'static str, HashMap<&'static str, &'static str>>,
}

impl I18n {
    pub fn new() -> Self {
        let mut t = HashMap::new();

        let mut en = HashMap::new();
        en.insert("setup_title", "Initial Setup");
        en.insert(
            "setup_welcome",
            "Welcome to AgroCore. Let's set up your system.",
        );
        en.insert("admin_password", "Admin Password");
        en.insert("tenant_name", "Organization Name");
        en.insert("tenant_slug", "Organization Slug (e.g. my-farm)");
        en.insert("finish_setup", "Complete Setup");
        en.insert("continue", "Continue");
        en.insert("wizard_language", "Wizard language");
        en.insert("setup_step_admin", "Admin");
        en.insert("setup_step_tenant", "Tenant");
        en.insert("setup_step_company", "Company");
        en.insert("setup_step_resources", "Resources");
        en.insert("setup_admin_title", "Administrator Account");
        en.insert(
            "setup_admin_desc",
            "Create the first admin user for this installation.",
        );
        en.insert("setup_tenant_title", "Create First Tenant");
        en.insert(
            "setup_tenant_desc",
            "A tenant represents your organization in the system.",
        );
        en.insert("setup_company_title", "Company Details");
        en.insert(
            "setup_company_desc",
            "Optional metadata for your organization.",
        );
        en.insert("company_name", "Company name");
        en.insert("company_address", "Company address");
        en.insert("company_country", "Country");
        en.insert("company_email", "Company email");
        en.insert("company_phone", "Company phone");
        en.insert("setup_resources_title", "Initial Resources");
        en.insert(
            "setup_resources_desc",
            "Define a starting field type for the workspace.",
        );
        en.insert("setup_resource_type", "Field type");
        en.insert("setup_equipment", "Equipment");
        en.insert("setup_first_machine", "First machine name");
        en.insert("redirecting", "Redirecting...");
        en.insert("dashboard", "Dashboard");
        en.insert("sites", "Sites");
        en.insert("tasks", "Tasks");
        en.insert("users", "Users");
        en.insert("settings", "Settings");
        en.insert("first_name", "First Name");
        en.insert("last_name", "Last Name");
        en.insert("email", "Email");
        en.insert("password", "Password");
        en.insert("company_profile", "Company profile");
        en.insert("tax_id", "Tax ID / VAT");
        en.insert("office_email", "Head office email");
        en.insert("international_phone", "International phone");
        en.insert("address", "Address");
        en.insert("website", "Website");
        en.insert("country", "Country");
        en.insert("update_profile", "Update profile");
        en.insert("display_and_language", "Display & language");
        en.insert("system_language", "System language");
        en.insert("language_de", "German");
        en.insert("language_en", "English");
        en.insert("language_es", "Spanish");
        en.insert("language_fr", "French");
        en.insert("language_pt", "Portuguese");
        en.insert("timezone", "Time zone");
        en.insert("timezone_europe_berlin", "Europe/Berlin (UTC+2)");
        en.insert("timezone_utc", "UTC");
        en.insert("system_status", "System status");
        en.insert("api_connection", "API connection");
        en.insert("online", "Online");
        en.insert("database", "Database");
        en.insert("connected", "Connected");
        en.insert("version", "Version");
        en.insert("export_system_log", "Export system log");
        en.insert("danger_zone", "Danger zone");
        en.insert(
            "danger_zone_desc",
            "These actions cannot be undone.",
        );
        en.insert("export_all_data", "Export all data");
        en.insert("delete_tenant", "Delete tenant");
        en.insert("profile_saved", "Profile saved locally.");
        en.insert(
            "validation_required_company",
            "Please fill in company name, address, and country.",
        );
        en.insert("time", "Time");
        en.insert("initialized", "Initialized");
        en.insert("mode_full", "Full View");
        en.insert("mode_simple", "Simple View");
        en.insert("wizard_tasks", "What would you like to do?");
        en.insert("start_wizard", "Start Wizard");
        en.insert("task_planting", "Plan Planting");
        en.insert("task_harvest", "Record Harvest");
        en.insert("task_protection", "Crop Protection");
        en.insert("livestock_management", "Livestock Management");
        en.insert("weather_and_phenology", "Weather & Phenology");
        en.insert("finance_and_pac", "Finance & PAC");
        en.insert("equipment_management", "Equipment Management");
        en.insert("analytics_and_predictions", "Analytics & Predictions");
        en.insert("site_management", "Site Management");
        en.insert("order_management", "Order Management");
        en.insert("add_animal", "Add Animal");
        en.insert("add_equipment", "Add Equipment");
        en.insert("record_treatment", "Record Treatment");
        en.insert("record_finance", "Record Finance");
        en.insert("weather_current", "Current Weather");
        en.insert("weather_stations", "Weather Stations");
        en.insert("phenology", "Phenology");
        en.insert("pac_applications", "PAC Applications");
        en.insert("cost_centers", "Cost Centers");
        en.insert("financial_records", "Financial Records");
        en.insert("theme_light", "Light");
        en.insert("theme_dark", "Dark");
        en.insert("theme_toggle", "Toggle Theme");
        en.insert("validation_required", "Please fill in all required fields.");
        en.insert("validation_invalid_email", "Please enter a valid email address.");
        en.insert("validation_invalid_phone", "Please enter a valid international phone number.");
        en.insert("validation_invalid_slug", "Please enter a valid slug.");
        en.insert("starting_setup", "Starting setup...");
        en.insert("first_machine_placeholder", "Optional machine name");
        en.insert("new_order_btn", "New Order");
        en.insert("new_order_form", "Create Order");
        en.insert("order_type", "Order type");
        en.insert("description", "Description");
        en.insert("actions", "Actions");
        en.insert("status", "Status");
        en.insert("site", "Site");
        en.insert("choose_site", "Choose a site");
        en.insert("cancel", "Cancel");
        en.insert("save", "Save");
        en.insert("login_title", "AgroCore Login");
        en.insert("login_subtitle", "Sign in with the first admin account.");
        en.insert("sign_in", "Sign in");
        en.insert("signing_in", "Signing in...");
        en.insert("role_simulate", "Simulate role:");
        en.insert("nav_dashboard", "Dashboard");
        en.insert("nav_wizard", "Start Wizard");
        en.insert("nav_sites", "Sites");
        en.insert("nav_tasks", "Tasks");
        en.insert("nav_map", "Map");
        en.insert("nav_livestock", "Livestock");
        en.insert("nav_weather", "Weather");
        en.insert("nav_resources", "Resources");
        en.insert("nav_equipment", "Equipment");
        en.insert("nav_finance", "Finance");
        en.insert("nav_analytics", "Analytics");
        en.insert("nav_compliance", "Compliance");
        en.insert("nav_users", "Users");
        en.insert("nav_settings", "Settings");
        en.insert("nav_grafana", "Grafana");
        en.insert("theme", "Theme");
        en.insert("logout", "Logout");
        en.insert("not_found", "404 Not Found");
        en.insert("dashboard_welcome_prefix", "Welcome to");
        en.insert("dashboard_simple_mode_text", "Use the guided assistants to finish complex work quickly.");
        en.insert("dashboard_tasks_desc", "Currently recorded");
        en.insert("dashboard_sites_desc", "Available parcels");
        en.insert("dashboard_users_desc", "Stored in the system");
        en.insert("dashboard_equipment_desc", "Available entries");
        en.insert("dashboard_weather_setup_address", "The setup address is used when available.");
        en.insert("active_tasks", "Active tasks");
        en.insert("no_active_tasks", "No active tasks.");
        en.insert("view_all", "View all");
        en.insert("quick_actions", "Quick actions");
        en.insert("issue_report", "Report issue");
        en.insert("weather_forecast", "Weather forecast");
        en.insert("no_weather_data", "No weather data available.");
        en.insert("no_sensor_weather", "No sensor available. The setup address is used when present.");
        en.insert("weather_no_external_data", "No weather data available. If there is no sensor, the setup address is used for an external weather lookup.");
        en.insert("weather_no_site_for_observation", "No site available for observation.");
        en.insert("weather_stations_title", "Weather stations");
        en.insert("no_weather_stations", "No weather stations available yet.");
        en.insert("phenology_title", "Phenology");
        en.insert("observe", "Observation");
        en.insert("last_report", "Last report");
        en.insert("report_observation", "Report observation");
        en.insert("observation_note", "Note");
        en.insert("weather_current_data", "Current weather data");
        en.insert("source", "Source");
        en.insert("temperature", "Temperature");
        en.insert("humidity", "Humidity");
        en.insert("wind", "Wind");
        en.insert("precipitation", "Precipitation");
        en.insert("current_weather", "Current weather");
        en.insert("station", "Station");
        en.insert("loading", "Loading...");
        en.insert("weather_observation_title", "Record BBCH observation");
        en.insert("weather_bbch_desc", "Current BBCH stages for your crops.");
        en.insert("resource_management", "Resource management");
        en.insert("resource_overview", "Overview of water, energy, supplies, and staff.");
        en.insert("history", "History");
        en.insert("add_inventory", "Add inventory");
        en.insert("water_irrigation", "Water & irrigation");
        en.insert("no_water_sensor", "No irrigation sensors connected yet.");
        en.insert("workers", "Workers");
        en.insert("in_use", "In use");
        en.insert("hours_today", "Hours today");
        en.insert("time_tracking_later", "Time tracking will follow later.");
        en.insert("workers_in_system", "Workers in the system");
        en.insert("inventory_stock", "Inventory stock");
        en.insert("inventory_available", "available equipment entries.");
        en.insert("resource_assignments_desc", "Assignments are derived from tasks and roles.");
        en.insert("livestock_total", "Total herd");
        en.insert("animals_in_system", "Animals in the system");
        en.insert("under_treatment", "Under treatment");
        en.insert("grazing", "On pasture");
        en.insert("treatment_from_data", "Calculated from treatment data.");
        en.insert("grazing_from_data", "Pasture status follows animal data.");
        en.insert("animal_list", "Animal list");
        en.insert("id_name", "ID / name");
        en.insert("species", "Species");
        en.insert("last_action", "Last action");
        en.insert("animal_actions", "Actions");
        en.insert("add_animal_btn", "Add animal");
        en.insert("add_treatment", "Add treatment");
        en.insert("no_animal_selected", "No animal selected.");
        en.insert("identifier", "Identifier");
        en.insert("breed", "Breed");
        en.insert("species_cattle", "Cattle");
        en.insert("species_sheep", "Sheep");
        en.insert("species_goat", "Goat");
        en.insert("species_pig", "Pig");
        en.insert("species_poultry", "Poultry");
        en.insert("species_horse", "Horse");
        en.insert("compliance_certification", "Compliance & certification");
        en.insert("manage_requirements", "Manage legal requirements and certifications.");
        en.insert("export_report", "Export report");
        en.insert("new_checklist", "New checklist");
        en.insert("compliance_score", "Compliance score");
        en.insert("open_actions", "Open actions");
        en.insert("next_audit", "Next audit");
        en.insert("certification_status", "Certification status");
        en.insert("application_history", "Plant protection history");
        en.insert("entry", "Entry");
        en.insert("recorded", "Recorded");
        en.insert("no_protection_entries", "No plant protection entries yet.");
        en.insert("create_checklist", "Create checklist");
        en.insert("due_date", "Due date");
        en.insert("type", "Type");
        en.insert("score_from_checks", "Calculated from inspections.");
        en.insert("active_checks", "Active checks");
        en.insert("next_audit_desc", "The date will be filled from audit data.");
        en.insert("certification_status_desc", "Certificates will appear after the audit data is connected.");
        en.insert("compliance_no_site_for_checklist", "No site available for a checklist.");
        en.insert("analytics_no_site_for_prediction", "No site available for the prediction.");
        en.insert("analytics_no_site_for_calculation", "No site available for the calculation.");
        en.insert("profitability_chart_placeholder", "[Chart: profitability per site]");
        en.insert("analytics_title", "Analytics & predictions");
        en.insert("harvest_prediction", "Harvest prediction");
        en.insert("based_on_weather", "Based on weather data and BBCH stage.");
        en.insert("confidence", "Confidence");
        en.insert("start_simulation", "Start simulation");
        en.insert("profitability", "Profitability");
        en.insert("profitability_analysis", "Analysis of margins per field.");
        en.insert("detail_report", "Detail report");
        en.insert("material_calculation", "Material calculation");
        en.insert("site_label", "Site");
        en.insert("measure", "Measure");
        en.insert("calculate_demand", "Calculate demand");
        en.insert("forecast_reference", "Merlot Block A");
        en.insert("operation_label", "Operation");
        en.insert("wizard_livestock", "Animal treatment");
        en.insert("wizard_finance", "Finance entry");
        en.insert("wizard_bbch", "BBCH observation");
        en.insert("wizard_title", "What would you like to do?");
        en.insert("wizard_more_tasks", "More tasks?");
        en.insert("wizard_explore", "Select a section to be guided step by step.");
        en.insert("wizard_start", "Start");
        en.insert("wizard_planting_desc", "Plan the next planting on your sites.");
        en.insert("wizard_protection_desc", "Document fertilization or crop protection.");
        en.insert("wizard_harvest_desc", "Record harvest quantities and quality.");
        en.insert("wizard_livestock_desc", "Document medical measures.");
        en.insert("wizard_finance_desc", "Quickly record income or expenses.");
        en.insert("wizard_bbch_desc", "Log current growth stages.");

        let mut de = HashMap::new();
        de.insert("setup_title", "Erstkonfiguration");
        de.insert(
            "setup_welcome",
            "Willkommen bei AgroCore. Lassen Sie uns Ihr System einrichten.",
        );
        de.insert("admin_password", "Admin-Passwort");
        de.insert("tenant_name", "Name der Organisation");
        de.insert("tenant_slug", "Kürzel (z.B. mein-betrieb)");
        de.insert("finish_setup", "Einrichtung abschließen");
        de.insert("continue", "Weiter");
        de.insert("wizard_language", "Sprache für den Assistenten");
        de.insert("setup_step_admin", "Admin");
        de.insert("setup_step_tenant", "Betrieb");
        de.insert("setup_step_company", "Firma");
        de.insert("setup_step_resources", "Ressourcen");
        de.insert("setup_admin_title", "Administrator-Konto");
        de.insert(
            "setup_admin_desc",
            "Legen Sie den ersten Admin-Benutzer für diese Installation an.",
        );
        de.insert("setup_tenant_title", "Ersten Betrieb anlegen");
        de.insert(
            "setup_tenant_desc",
            "Der Betrieb bildet Ihre Organisation im System ab.",
        );
        de.insert("setup_company_title", "Firmendaten");
        de.insert(
            "setup_company_desc",
            "Optionale Metadaten für Ihre Organisation.",
        );
        de.insert("company_name", "Firmenname");
        de.insert("company_address", "Firmenadresse");
        de.insert("company_country", "Land");
        de.insert("company_email", "Firmen-E-Mail");
        de.insert("company_phone", "Firmentelefon");
        de.insert("setup_resources_title", "Erste Ressourcen");
        de.insert(
            "setup_resources_desc",
            "Legen Sie einen Starttyp für die Flächen fest.",
        );
        de.insert("setup_resource_type", "Flächentyp");
        de.insert("setup_equipment", "Equipment");
        de.insert("setup_first_machine", "Erste Maschine");
        de.insert("redirecting", "Weiterleitung...");
        de.insert("dashboard", "Dashboard");
        de.insert("sites", "Standorte");
        de.insert("tasks", "Aufgaben");
        de.insert("users", "Benutzer");
        de.insert("settings", "Einstellungen");
        de.insert("first_name", "Vorname");
        de.insert("last_name", "Nachname");
        de.insert("email", "E-Mail");
        de.insert("password", "Passwort");
        de.insert("company_profile", "Unternehmensprofil");
        de.insert("tax_id", "Steuernummer / VAT");
        de.insert("office_email", "E-Mail (Zentrale)");
        de.insert("international_phone", "Telefon international");
        de.insert("address", "Adresse");
        de.insert("website", "Webseite");
        de.insert("country", "Land");
        de.insert("update_profile", "Profil aktualisieren");
        de.insert("display_and_language", "Anzeige & Sprache");
        de.insert("system_language", "System-Sprache");
        de.insert("language_de", "Deutsch");
        de.insert("language_en", "Englisch");
        de.insert("language_es", "Spanisch");
        de.insert("language_fr", "Französisch");
        de.insert("language_pt", "Portugiesisch");
        de.insert("timezone", "Zeitzone");
        de.insert("timezone_europe_berlin", "Europa/Berlin (UTC+2)");
        de.insert("timezone_utc", "UTC");
        de.insert("system_status", "System-Status");
        de.insert("api_connection", "API Verbindung");
        de.insert("online", "Online");
        de.insert("database", "Datenbank");
        de.insert("connected", "Verbunden");
        de.insert("version", "Version");
        de.insert("export_system_log", "System-Log exportieren");
        de.insert("danger_zone", "Gefahrenzone");
        de.insert(
            "danger_zone_desc",
            "Diese Aktionen können nicht rückgängig gemacht werden.",
        );
        de.insert("export_all_data", "Alle Daten exportieren");
        de.insert("delete_tenant", "Tenant löschen");
        de.insert("profile_saved", "Profil lokal gespeichert.");
        de.insert(
            "validation_required_company",
            "Bitte Firmenname, Adresse und Land ausfüllen.",
        );
        de.insert("time", "Zeit");
        de.insert("initialized", "Initialisiert");
        de.insert("mode_full", "Experten-Ansicht");
        de.insert("mode_simple", "Einfache Ansicht");
        de.insert("wizard_tasks", "Was möchten Sie tun?");
        de.insert("start_wizard", "Assistent starten");
        de.insert("task_planting", "Aussaat planen");
        de.insert("task_harvest", "Ernte erfassen");
        de.insert("task_protection", "Pflanzenschutz");
        de.insert("livestock_management", "Tierhaltung");
        de.insert("weather_and_phenology", "Wetter & Phänologie");
        de.insert("finance_and_pac", "Finanzen & PAC");
        de.insert("equipment_management", "Maschinen & Geräte");
        de.insert("analytics_and_predictions", "Analyse & Vorhersage");
        de.insert("site_management", "Standortverwaltung");
        de.insert("order_management", "Auftragsmanagement");
        de.insert("add_animal", "Tier hinzufügen");
        de.insert("add_equipment", "Gerät hinzufügen");
        de.insert("record_treatment", "Behandlung erfassen");
        de.insert("record_finance", "Buchung erstellen");
        de.insert("weather_current", "Aktuelles Wetter");
        de.insert("weather_stations", "Wetterstationen");
        de.insert("phenology", "Phänologie");
        de.insert("pac_applications", "PAC-Anträge");
        de.insert("cost_centers", "Kostenstellen");
        de.insert("financial_records", "Finanzaufzeichnungen");
        de.insert("theme_light", "Hell");
        de.insert("theme_dark", "Dunkel");
        de.insert("theme_toggle", "Farbmodus umschalten");
        de.insert("validation_required", "Bitte alle Pflichtfelder ausfüllen.");
        de.insert("validation_invalid_email", "Bitte eine gültige E-Mail-Adresse eingeben.");
        de.insert("validation_invalid_phone", "Bitte eine gültige internationale Telefonnummer eingeben.");
        de.insert("validation_invalid_slug", "Bitte ein gültiges Kürzel eingeben.");
        de.insert("starting_setup", "Setup wird gestartet...");
        de.insert("first_machine_placeholder", "Optionale Maschinenbezeichnung");
        de.insert("new_order_btn", "Neuer Auftrag");
        de.insert("new_order_form", "Auftrag anlegen");
        de.insert("order_type", "Auftragstyp");
        de.insert("description", "Beschreibung");
        de.insert("actions", "Aktionen");
        de.insert("status", "Status");
        de.insert("site", "Fläche");
        de.insert("choose_site", "Fläche wählen");
        de.insert("cancel", "Abbrechen");
        de.insert("save", "Speichern");
        de.insert("login_title", "AgroCore Login");
        de.insert("login_subtitle", "Mit dem ersten Admin-Konto anmelden.");
        de.insert("sign_in", "Anmelden");
        de.insert("signing_in", "Anmeldung läuft...");
        de.insert("role_simulate", "Rolle simulieren:");
        de.insert("nav_dashboard", "Dashboard");
        de.insert("nav_wizard", "Assistent starten");
        de.insert("nav_sites", "Flächen");
        de.insert("nav_tasks", "Aufträge");
        de.insert("nav_map", "Karte");
        de.insert("nav_livestock", "Tierhaltung");
        de.insert("nav_weather", "Wetter");
        de.insert("nav_resources", "Ressourcen");
        de.insert("nav_equipment", "Equipment");
        de.insert("nav_finance", "Finanzen");
        de.insert("nav_analytics", "Analytics");
        de.insert("nav_compliance", "Compliance");
        de.insert("nav_users", "Benutzer");
        de.insert("nav_settings", "Einstellungen");
        de.insert("nav_grafana", "Grafana");
        de.insert("theme", "Theme");
        de.insert("logout", "Abmelden");
        de.insert("not_found", "404 Nicht gefunden");
        de.insert("dashboard_welcome_prefix", "Willkommen bei");
        de.insert("dashboard_simple_mode_text", "Nutzen Sie unsere Assistenten, um komplexe Aufgaben einfach und schnell zu erledigen.");
        de.insert("dashboard_tasks_desc", "Aktuell erfasst");
        de.insert("dashboard_sites_desc", "Verfügbare Parzellen");
        de.insert("dashboard_users_desc", "Im System hinterlegt");
        de.insert("dashboard_equipment_desc", "Verfügbare Einträge");
        de.insert("dashboard_weather_setup_address", "Adresse aus dem Setup wird verwendet, wenn vorhanden.");
        de.insert("active_tasks", "Aktive Aufträge");
        de.insert("no_active_tasks", "Keine aktiven Aufträge.");
        de.insert("view_all", "Alle ansehen");
        de.insert("quick_actions", "Schnellaktionen");
        de.insert("issue_report", "Störfall melden");
        de.insert("weather_forecast", "Wettervorhersage");
        de.insert("no_weather_data", "Keine Wetterdaten verfügbar.");
        de.insert("no_sensor_weather", "Kein Sensor vorhanden. Wenn vorhanden, wird die Adresse aus dem Setup verwendet.");
        de.insert("weather_no_external_data", "Keine Wetterdaten verfügbar. Wenn keine Station vorhanden ist, wird die Adresse aus dem Setup für eine externe Wetterabfrage genutzt.");
        de.insert("weather_no_site_for_observation", "Keine Fläche für die Beobachtung verfügbar.");
        de.insert("weather_stations_title", "Wetterstationen");
        de.insert("no_weather_stations", "Noch keine Wetterstationen angelegt.");
        de.insert("phenology_title", "Phänologie");
        de.insert("observe", "Beobachtung");
        de.insert("last_report", "Letzte Meldung");
        de.insert("report_observation", "Beobachtung melden");
        de.insert("observation_note", "Hinweis");
        de.insert("weather_current_data", "Aktuelle Wetterdaten");
        de.insert("source", "Quelle");
        de.insert("temperature", "Temperatur");
        de.insert("humidity", "Luftfeuchte");
        de.insert("wind", "Wind");
        de.insert("precipitation", "Niederschlag");
        de.insert("current_weather", "Aktuelles Wetter");
        de.insert("station", "Station");
        de.insert("loading", "Lädt...");
        de.insert("weather_observation_title", "BBCH-Beobachtung erfassen");
        de.insert("weather_bbch_desc", "Aktuelle BBCH-Stadien Ihrer Kulturen.");
        de.insert("resource_management", "Ressourcenverwaltung");
        de.insert("resource_overview", "Überblick über Wasser, Energie, Betriebsmittel und Personal.");
        de.insert("history", "Verlauf");
        de.insert("add_inventory", "Inventar hinzufügen");
        de.insert("water_irrigation", "Wasser & Bewässerung");
        de.insert("no_water_sensor", "Noch keine Bewässerungssensoren verbunden.");
        de.insert("workers", "Arbeitskräfte");
        de.insert("in_use", "Im Einsatz");
        de.insert("hours_today", "Stunden heute");
        de.insert("time_tracking_later", "Zeiterfassung folgt später.");
        de.insert("workers_in_system", "Benutzer im System");
        de.insert("inventory_stock", "Lagerbestand");
        de.insert("inventory_available", "Equipment-Einträge sind verfügbar.");
        de.insert("resource_assignments_desc", "Arbeitszuweisungen werden aus Aufträgen und Rollen abgeleitet.");
        de.insert("livestock_total", "Gesamtbestand");
        de.insert("animals_in_system", "Tiere im System");
        de.insert("under_treatment", "In Behandlung");
        de.insert("grazing", "Auf Weide");
        de.insert("treatment_from_data", "Wird aus Behandlungsdaten berechnet.");
        de.insert("grazing_from_data", "Weidestatus folgt aus Tierdaten.");
        de.insert("animal_list", "Tierliste");
        de.insert("id_name", "ID / Name");
        de.insert("species", "Art");
        de.insert("last_action", "Letzte Aktion");
        de.insert("animal_actions", "Aktionen");
        de.insert("add_animal_btn", "Tier hinzufügen");
        de.insert("add_treatment", "Behandlung hinzufügen");
        de.insert("no_animal_selected", "Kein Tier ausgewählt.");
        de.insert("identifier", "Kennung");
        de.insert("breed", "Rasse");
        de.insert("species_cattle", "Rind");
        de.insert("species_sheep", "Schaf");
        de.insert("species_goat", "Ziege");
        de.insert("species_pig", "Schwein");
        de.insert("species_poultry", "Geflügel");
        de.insert("species_horse", "Pferd");
        de.insert("compliance_certification", "Compliance & Zertifizierung");
        de.insert("manage_requirements", "Verwalten Sie Ihre gesetzlichen Anforderungen und Zertifizierungen.");
        de.insert("export_report", "Bericht exportieren");
        de.insert("new_checklist", "Neue Prüfung");
        de.insert("compliance_score", "Compliance Score");
        de.insert("open_actions", "Offene Maßnahmen");
        de.insert("next_audit", "Nächstes Audit");
        de.insert("certification_status", "Zertifizierungs-Status");
        de.insert("application_history", "Anwendungshistorie");
        de.insert("entry", "Eintrag");
        de.insert("recorded", "Erfasst");
        de.insert("no_protection_entries", "Noch keine Pflanzenschutz-Einträge vorhanden.");
        de.insert("create_checklist", "Prüfung anlegen");
        de.insert("due_date", "Fällig am");
        de.insert("type", "Typ");
        de.insert("score_from_checks", "Wird aus Prüfungen berechnet.");
        de.insert("active_checks", "Aktive Prüfungen");
        de.insert("next_audit_desc", "Termin wird aus Auditdaten gefüllt.");
        de.insert("certification_status_desc", "Zertifikate werden nach Anbindung der Prüfungsdaten angezeigt.");
        de.insert("compliance_no_site_for_checklist", "Keine Fläche für eine Prüfung verfügbar.");
        de.insert("analytics_no_site_for_prediction", "Keine Fläche für die Vorhersage verfügbar.");
        de.insert("analytics_no_site_for_calculation", "Keine Fläche für die Berechnung verfügbar.");
        de.insert("profitability_chart_placeholder", "[Graph: Profitabilität pro Standort]");
        de.insert("analytics_title", "Analytics & Vorhersagen");
        de.insert("harvest_prediction", "Erntevorhersage");
        de.insert("based_on_weather", "Basierend auf Wetterdaten und BBCH-Stadium.");
        de.insert("confidence", "Konfidenz");
        de.insert("start_simulation", "Simulation starten");
        de.insert("profitability", "Wirtschaftlichkeit");
        de.insert("profitability_analysis", "Analyse der Deckungsbeiträge pro Schlag.");
        de.insert("detail_report", "Detailbericht");
        de.insert("material_calculation", "Materialkalkulation");
        de.insert("site_label", "Standort");
        de.insert("measure", "Maßnahme");
        de.insert("calculate_demand", "Bedarf berechnen");
        de.insert("forecast_reference", "Merlot Block A");
        de.insert("operation_label", "Vorgang");
        de.insert("wizard_livestock", "Tierbehandlung");
        de.insert("wizard_finance", "Finanz-Buchung");
        de.insert("wizard_bbch", "BBCH-Beobachtung");
        de.insert("wizard_title", "Was möchten Sie tun?");
        de.insert("wizard_more_tasks", "Weitere Aufgaben?");
        de.insert("wizard_explore", "Wählen Sie einen Bereich aus, um Schritt für Schritt geführt zu werden.");
        de.insert("wizard_start", "Starten");
        de.insert("wizard_planting_desc", "Planen Sie die nächste Aussaat auf Ihren Flächen.");
        de.insert("wizard_protection_desc", "Dokumentieren Sie Düngung oder Pflanzenschutz.");
        de.insert("wizard_harvest_desc", "Erfassen Sie Erntemengen und Qualität.");
        de.insert("wizard_livestock_desc", "Dokumentieren Sie medizinische Maßnahmen.");
        de.insert("wizard_finance_desc", "Einnahmen oder Ausgaben schnell erfassen.");
        de.insert("wizard_bbch_desc", "Aktuelle Wachstumsstadien protokollieren.");

        let mut es = HashMap::new();
        es.insert("setup_title", "Configuración Inicial");
        es.insert(
            "setup_welcome",
            "Bienvenido a AgroCore. Vamos a configurar su sistema.",
        );
        es.insert("admin_password", "Contraseña de Administrador");
        es.insert("tenant_name", "Nombre de la Organización");
        es.insert("tenant_slug", "Identificador (ej. mi-granja)");
        es.insert("finish_setup", "Completar Configuración");
        es.insert("continue", "Continuar");
        es.insert("wizard_language", "Idioma del asistente");
        es.insert("setup_step_admin", "Admin");
        es.insert("setup_step_tenant", "Empresa");
        es.insert("setup_step_company", "Empresa");
        es.insert("setup_step_resources", "Recursos");
        es.insert("setup_admin_title", "Cuenta de administrador");
        es.insert(
            "setup_admin_desc",
            "Cree el primer usuario administrador para esta instalación.",
        );
        es.insert("setup_tenant_title", "Crear primera empresa");
        es.insert(
            "setup_tenant_desc",
            "La empresa representa su organización en el sistema.",
        );
        es.insert("setup_company_title", "Datos de la empresa");
        es.insert(
            "setup_company_desc",
            "Metadatos opcionales de su organización.",
        );
        es.insert("company_name", "Nombre de la empresa");
        es.insert("company_address", "Dirección de la empresa");
        es.insert("company_country", "País");
        es.insert("company_email", "Correo de la empresa");
        es.insert("company_phone", "Teléfono de la empresa");
        es.insert("setup_resources_title", "Recursos iniciales");
        es.insert(
            "setup_resources_desc",
            "Defina un tipo inicial de campo para el espacio de trabajo.",
        );
        es.insert("setup_resource_type", "Tipo de campo");
        es.insert("setup_equipment", "Equipamiento");
        es.insert("setup_first_machine", "Primer equipo");
        es.insert("redirecting", "Redirigiendo...");
        es.insert("dashboard", "Panel de Control");
        es.insert("sites", "Sitios");
        es.insert("tasks", "Tareas");
        es.insert("users", "Usuarios");
        es.insert("settings", "Ajustes");
        es.insert("first_name", "Nombre");
        es.insert("last_name", "Apellido");
        es.insert("email", "Correo electrónico");
        es.insert("password", "Contraseña");
        es.insert("company_profile", "Perfil de la empresa");
        es.insert("tax_id", "NIF / IVA");
        es.insert("office_email", "Correo de la central");
        es.insert("international_phone", "Teléfono internacional");
        es.insert("address", "Dirección");
        es.insert("website", "Sitio web");
        es.insert("country", "País");
        es.insert("update_profile", "Actualizar perfil");
        es.insert("display_and_language", "Pantalla e idioma");
        es.insert("system_language", "Idioma del sistema");
        es.insert("language_de", "Alemán");
        es.insert("language_en", "Inglés");
        es.insert("language_es", "Español");
        es.insert("language_fr", "Francés");
        es.insert("language_pt", "Portugués");
        es.insert("timezone", "Zona horaria");
        es.insert("timezone_europe_berlin", "Europa/Berlín (UTC+2)");
        es.insert("timezone_utc", "UTC");
        es.insert("system_status", "Estado del sistema");
        es.insert("api_connection", "Conexión API");
        es.insert("online", "En línea");
        es.insert("database", "Base de datos");
        es.insert("connected", "Conectada");
        es.insert("version", "Versión");
        es.insert("export_system_log", "Exportar registro del sistema");
        es.insert("danger_zone", "Zona de riesgo");
        es.insert("danger_zone_desc", "Estas acciones no se pueden deshacer.");
        es.insert("export_all_data", "Exportar todos los datos");
        es.insert("delete_tenant", "Eliminar tenant");
        es.insert("profile_saved", "Perfil guardado localmente.");
        es.insert("validation_required_company", "Complete el nombre de la empresa, la dirección y el país.");
        es.insert("time", "Hora");
        es.insert("initialized", "Inicializado");
        es.insert("dashboard_simple_mode_text", "Use los asistentes guiados para terminar tareas complejas rápidamente.");
        es.insert("dashboard_tasks_desc", "Registrado actualmente");
        es.insert("dashboard_sites_desc", "Parcelas disponibles");
        es.insert("dashboard_users_desc", "Guardados en el sistema");
        es.insert("dashboard_equipment_desc", "Entradas disponibles");
        es.insert("dashboard_weather_setup_address", "Se usa la dirección de la configuración cuando está disponible.");
        es.insert("weather_no_external_data", "No hay datos meteorológicos. Si no existe una estación, se usa la dirección de configuración para una consulta externa.");
        es.insert("weather_no_site_for_observation", "No hay una parcela disponible para la observación.");
        es.insert("weather_observation_title", "Registrar observación BBCH");
        es.insert("weather_bbch_desc", "Estados BBCH actuales de sus cultivos.");
        es.insert("resource_assignments_desc", "Las asignaciones se derivan de tareas y roles.");
        es.insert("workers_in_system", "Usuarios en el sistema");
        es.insert("treatment_from_data", "Calculado a partir de los datos de tratamiento.");
        es.insert("grazing_from_data", "El estado de pastoreo sigue los datos del animal.");
        es.insert("identifier", "Identificador");
        es.insert("breed", "Raza");
        es.insert("species_cattle", "Bovino");
        es.insert("species_sheep", "Oveja");
        es.insert("species_goat", "Cabra");
        es.insert("species_pig", "Cerdo");
        es.insert("species_poultry", "Aves");
        es.insert("species_horse", "Caballo");
        es.insert("score_from_checks", "Calculado a partir de inspecciones.");
        es.insert("active_checks", "Inspecciones activas");
        es.insert("next_audit_desc", "La fecha se rellena con los datos de auditoría.");
        es.insert("certification_status_desc", "Los certificados aparecerán cuando se conecten los datos de auditoría.");
        es.insert("compliance_no_site_for_checklist", "No hay una parcela disponible para la lista de verificación.");
        es.insert("analytics_no_site_for_prediction", "No hay una parcela disponible para la predicción.");
        es.insert("analytics_no_site_for_calculation", "No hay una parcela disponible para el cálculo.");
        es.insert("profitability_chart_placeholder", "[Gráfico: rentabilidad por parcela]");
        es.insert("wizard_livestock", "Tratamiento animal");
        es.insert("wizard_finance", "Entrada financiera");
        es.insert("wizard_bbch", "Observación BBCH");

        let mut fr = HashMap::new();
        fr.insert("setup_title", "Configuration Initiale");
        fr.insert(
            "setup_welcome",
            "Bienvenue sur AgroCore. Configurer votre système.",
        );
        fr.insert("admin_password", "Mot de passe Administrateur");
        fr.insert("tenant_name", "Nom de l'organisation");
        fr.insert("tenant_slug", "Identifiant (ex. ma-ferme)");
        fr.insert("finish_setup", "Terminer la configuration");
        fr.insert("continue", "Continuer");
        fr.insert("wizard_language", "Langue de l'assistant");
        fr.insert("setup_step_admin", "Admin");
        fr.insert("setup_step_tenant", "Entreprise");
        fr.insert("setup_step_company", "Entreprise");
        fr.insert("setup_step_resources", "Ressources");
        fr.insert("setup_admin_title", "Compte administrateur");
        fr.insert(
            "setup_admin_desc",
            "Créez le premier utilisateur administrateur pour cette installation.",
        );
        fr.insert("setup_tenant_title", "Créer la première entreprise");
        fr.insert(
            "setup_tenant_desc",
            "L'entreprise représente votre organisation dans le système.",
        );
        fr.insert("setup_company_title", "Détails de l'entreprise");
        fr.insert(
            "setup_company_desc",
            "Métadonnées facultatives pour votre organisation.",
        );
        fr.insert("company_name", "Nom de l'entreprise");
        fr.insert("company_address", "Adresse de l'entreprise");
        fr.insert("company_country", "Pays");
        fr.insert("company_email", "E-mail de l'entreprise");
        fr.insert("company_phone", "Téléphone de l'entreprise");
        fr.insert("setup_resources_title", "Ressources initiales");
        fr.insert(
            "setup_resources_desc",
            "Définissez un type de champ de départ pour l'espace de travail.",
        );
        fr.insert("setup_resource_type", "Type de champ");
        fr.insert("setup_equipment", "Équipement");
        fr.insert("setup_first_machine", "Première machine");
        fr.insert("redirecting", "Redirection...");
        fr.insert("dashboard", "Tableau de bord");
        fr.insert("sites", "Sites");
        fr.insert("tasks", "Tâches");
        fr.insert("users", "Utilisateurs");
        fr.insert("settings", "Paramètres");
        fr.insert("first_name", "Prénom");
        fr.insert("last_name", "Nom");
        fr.insert("email", "E-mail");
        fr.insert("password", "Mot de passe");
        fr.insert("company_profile", "Profil de l'entreprise");
        fr.insert("tax_id", "NIF / TVA");
        fr.insert("office_email", "E-mail du siège");
        fr.insert("international_phone", "Téléphone international");
        fr.insert("address", "Adresse");
        fr.insert("website", "Site web");
        fr.insert("country", "Pays");
        fr.insert("update_profile", "Mettre à jour le profil");
        fr.insert("display_and_language", "Affichage et langue");
        fr.insert("system_language", "Langue du système");
        fr.insert("language_de", "Allemand");
        fr.insert("language_en", "Anglais");
        fr.insert("language_es", "Espagnol");
        fr.insert("language_fr", "Français");
        fr.insert("language_pt", "Portugais");
        fr.insert("timezone", "Fuseau horaire");
        fr.insert("timezone_europe_berlin", "Europe/Berlin (UTC+2)");
        fr.insert("timezone_utc", "UTC");
        fr.insert("system_status", "État du système");
        fr.insert("api_connection", "Connexion API");
        fr.insert("online", "En ligne");
        fr.insert("database", "Base de données");
        fr.insert("connected", "Connectée");
        fr.insert("version", "Version");
        fr.insert("export_system_log", "Exporter le journal système");
        fr.insert("danger_zone", "Zone dangereuse");
        fr.insert("danger_zone_desc", "Ces actions sont irréversibles.");
        fr.insert("export_all_data", "Exporter toutes les données");
        fr.insert("delete_tenant", "Supprimer le tenant");
        fr.insert("profile_saved", "Profil enregistré localement.");
        fr.insert("validation_required_company", "Veuillez renseigner le nom de l'entreprise, l'adresse et le pays.");
        fr.insert("time", "Heure");
        fr.insert("initialized", "Initialisé");
        fr.insert("dashboard_simple_mode_text", "Utilisez les assistants guidés pour terminer rapidement les tâches complexes.");
        fr.insert("dashboard_tasks_desc", "Actuellement enregistré");
        fr.insert("dashboard_sites_desc", "Parcelles disponibles");
        fr.insert("dashboard_users_desc", "Enregistrés dans le système");
        fr.insert("dashboard_equipment_desc", "Entrées disponibles");
        fr.insert("dashboard_weather_setup_address", "L'adresse de configuration est utilisée si elle existe.");
        fr.insert("weather_no_external_data", "Aucune donnée météo. S'il n'y a pas de station, l'adresse de configuration est utilisée pour une requête externe.");
        fr.insert("weather_no_site_for_observation", "Aucune parcelle disponible pour l'observation.");
        fr.insert("weather_observation_title", "Enregistrer une observation BBCH");
        fr.insert("weather_bbch_desc", "Stades BBCH actuels de vos cultures.");
        fr.insert("resource_assignments_desc", "Les affectations sont dérivées des tâches et des rôles.");
        fr.insert("workers_in_system", "Utilisateurs dans le système");
        fr.insert("treatment_from_data", "Calculé à partir des données de traitement.");
        fr.insert("grazing_from_data", "L'état de pâturage suit les données animales.");
        fr.insert("identifier", "Identifiant");
        fr.insert("breed", "Race");
        fr.insert("species_cattle", "Bovin");
        fr.insert("species_sheep", "Mouton");
        fr.insert("species_goat", "Chèvre");
        fr.insert("species_pig", "Porc");
        fr.insert("species_poultry", "Volaille");
        fr.insert("species_horse", "Cheval");
        fr.insert("score_from_checks", "Calculé à partir des contrôles.");
        fr.insert("active_checks", "Contrôles actifs");
        fr.insert("next_audit_desc", "La date sera remplie à partir des données d'audit.");
        fr.insert("certification_status_desc", "Les certificats apparaîtront une fois les données d'audit connectées.");
        fr.insert("compliance_no_site_for_checklist", "Aucune parcelle disponible pour la liste de contrôle.");
        fr.insert("analytics_no_site_for_prediction", "Aucune parcelle disponible pour la prédiction.");
        fr.insert("analytics_no_site_for_calculation", "Aucune parcelle disponible pour le calcul.");
        fr.insert("profitability_chart_placeholder", "[Graphique : rentabilité par parcelle]");
        fr.insert("wizard_livestock", "Traitement animal");
        fr.insert("wizard_finance", "Entrée financière");
        fr.insert("wizard_bbch", "Observation BBCH");

        let mut pt = HashMap::new();
        pt.insert("setup_title", "Configuração Inicial");
        pt.insert(
            "setup_welcome",
            "Bem-vindo ao AgroCore. Vamos configurar o seu sistema.",
        );
        pt.insert("admin_password", "Senha do Administrador");
        pt.insert("tenant_name", "Nome da Organização");
        pt.insert("tenant_slug", "Identificador (ex: minha-fazenda)");
        pt.insert("finish_setup", "Concluir Configuração");
        pt.insert("continue", "Continuar");
        pt.insert("wizard_language", "Idioma do assistente");
        pt.insert("setup_step_admin", "Admin");
        pt.insert("setup_step_tenant", "Empresa");
        pt.insert("setup_step_company", "Empresa");
        pt.insert("setup_step_resources", "Recursos");
        pt.insert("setup_admin_title", "Conta de administrador");
        pt.insert(
            "setup_admin_desc",
            "Crie o primeiro usuário administrador para esta instalação.",
        );
        pt.insert("setup_tenant_title", "Criar primeira empresa");
        pt.insert(
            "setup_tenant_desc",
            "A empresa representa a sua organização no sistema.",
        );
        pt.insert("setup_company_title", "Detalhes da empresa");
        pt.insert(
            "setup_company_desc",
            "Metadados opcionais da sua organização.",
        );
        pt.insert("company_name", "Nome da empresa");
        pt.insert("company_address", "Endereço da empresa");
        pt.insert("company_country", "País");
        pt.insert("company_email", "E-mail da empresa");
        pt.insert("company_phone", "Telefone da empresa");
        pt.insert("setup_resources_title", "Recursos iniciais");
        pt.insert(
            "setup_resources_desc",
            "Defina um tipo inicial de campo para o espaço de trabalho.",
        );
        pt.insert("setup_resource_type", "Tipo de campo");
        pt.insert("setup_equipment", "Equipamento");
        pt.insert("setup_first_machine", "Primeira máquina");
        pt.insert("redirecting", "A redirecionar...");
        pt.insert("dashboard", "Painel");
        pt.insert("sites", "Locais");
        pt.insert("tasks", "Tarefas");
        pt.insert("users", "Usuários");
        pt.insert("settings", "Configurações");
        pt.insert("first_name", "Nome");
        pt.insert("last_name", "Sobrenome");
        pt.insert("email", "E-mail");
        pt.insert("password", "Palavra-passe");
        pt.insert("company_profile", "Perfil da empresa");
        pt.insert("tax_id", "NIF / IVA");
        pt.insert("office_email", "E-mail da sede");
        pt.insert("international_phone", "Telefone internacional");
        pt.insert("address", "Endereço");
        pt.insert("website", "Site");
        pt.insert("country", "País");
        pt.insert("update_profile", "Atualizar perfil");
        pt.insert("display_and_language", "Exibição e idioma");
        pt.insert("system_language", "Idioma do sistema");
        pt.insert("language_de", "Alemão");
        pt.insert("language_en", "Inglês");
        pt.insert("language_es", "Espanhol");
        pt.insert("language_fr", "Francês");
        pt.insert("language_pt", "Português");
        pt.insert("timezone", "Fuso horário");
        pt.insert("timezone_europe_berlin", "Europa/Berlim (UTC+2)");
        pt.insert("timezone_utc", "UTC");
        pt.insert("system_status", "Estado do sistema");
        pt.insert("api_connection", "Ligação API");
        pt.insert("online", "Online");
        pt.insert("database", "Base de dados");
        pt.insert("connected", "Ligada");
        pt.insert("version", "Versão");
        pt.insert("export_system_log", "Exportar registo do sistema");
        pt.insert("danger_zone", "Zona de risco");
        pt.insert("danger_zone_desc", "Estas ações não podem ser desfeitas.");
        pt.insert("export_all_data", "Exportar todos os dados");
        pt.insert("delete_tenant", "Eliminar tenant");
        pt.insert("profile_saved", "Perfil guardado localmente.");
        pt.insert("validation_required_company", "Preencha o nome da empresa, o endereço e o país.");
        pt.insert("time", "Hora");
        pt.insert("initialized", "Inicializado");
        pt.insert("dashboard_simple_mode_text", "Use os assistentes guiados para terminar tarefas complexas rapidamente.");
        pt.insert("dashboard_tasks_desc", "Registado atualmente");
        pt.insert("dashboard_sites_desc", "Parcelas disponíveis");
        pt.insert("dashboard_users_desc", "Guardado no sistema");
        pt.insert("dashboard_equipment_desc", "Entradas disponíveis");
        pt.insert("dashboard_weather_setup_address", "O endereço da configuração é usado quando disponível.");
        pt.insert("weather_no_external_data", "Sem dados meteorológicos. Se não houver estação, usa-se o endereço da configuração para uma consulta externa.");
        pt.insert("weather_no_site_for_observation", "Não existe uma parcela disponível para a observação.");
        pt.insert("weather_observation_title", "Registar observação BBCH");
        pt.insert("weather_bbch_desc", "Estádios BBCH atuais das suas culturas.");
        pt.insert("resource_assignments_desc", "As atribuições são derivadas de tarefas e funções.");
        pt.insert("workers_in_system", "Utilizadores no sistema");
        pt.insert("treatment_from_data", "Calculado a partir dos dados de tratamento.");
        pt.insert("grazing_from_data", "O estado de pastoreio segue os dados do animal.");
        pt.insert("identifier", "Identificador");
        pt.insert("breed", "Raça");
        pt.insert("species_cattle", "Bovino");
        pt.insert("species_sheep", "Ovelha");
        pt.insert("species_goat", "Cabra");
        pt.insert("species_pig", "Porco");
        pt.insert("species_poultry", "Aves");
        pt.insert("species_horse", "Cavalo");
        pt.insert("score_from_checks", "Calculado com base nas inspeções.");
        pt.insert("active_checks", "Inspeções ativas");
        pt.insert("next_audit_desc", "A data será preenchida com dados de auditoria.");
        pt.insert("certification_status_desc", "Os certificados aparecem quando os dados de auditoria estiverem ligados.");
        pt.insert("compliance_no_site_for_checklist", "Não há uma parcela disponível para a lista de verificação.");
        pt.insert("analytics_no_site_for_prediction", "Não há uma parcela disponível para a previsão.");
        pt.insert("analytics_no_site_for_calculation", "Não há uma parcela disponível para o cálculo.");
        pt.insert("profitability_chart_placeholder", "[Gráfico: rentabilidade por parcela]");
        pt.insert("wizard_livestock", "Tratamento animal");
        pt.insert("wizard_finance", "Entrada financeira");
        pt.insert("wizard_bbch", "Observação BBCH");

        t.insert("en", en);
        t.insert("de", de);
        t.insert("es", es);
        t.insert("fr", fr);
        t.insert("pt", pt);

        Self { translations: t }
    }

    pub fn t(&self, lang: &str, key: &str) -> String {
        self.translations
            .get(lang)
            .and_then(|m| m.get(key))
            .map(|s| s.to_string())
            .unwrap_or_else(|| key.to_string())
    }
}
