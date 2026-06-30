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
        en.insert("setup_welcome", "Welcome to AgroCore. Let's set up your system.");
        en.insert("admin_password", "Admin Password");
        en.insert("tenant_name", "Organization Name");
        en.insert("tenant_slug", "Organization Slug (e.g. my-farm)");
        en.insert("finish_setup", "Complete Setup");
        en.insert("dashboard", "Dashboard");
        en.insert("sites", "Sites");
        en.insert("tasks", "Tasks");
        en.insert("users", "Users");
        en.insert("settings", "Settings");
        en.insert("first_name", "First Name");
        en.insert("last_name", "Last Name");
        en.insert("email", "Email");
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

        let mut de = HashMap::new();
        de.insert("setup_title", "Erstkonfiguration");
        de.insert("setup_welcome", "Willkommen bei AgroCore. Lassen Sie uns Ihr System einrichten.");
        de.insert("admin_password", "Admin-Passwort");
        de.insert("tenant_name", "Name der Organisation");
        de.insert("tenant_slug", "Kürzel (z.B. mein-betrieb)");
        de.insert("finish_setup", "Einrichtung abschließen");
        de.insert("dashboard", "Dashboard");
        de.insert("sites", "Standorte");
        de.insert("tasks", "Aufgaben");
        de.insert("users", "Benutzer");
        de.insert("settings", "Einstellungen");
        de.insert("first_name", "Vorname");
        de.insert("last_name", "Nachname");
        de.insert("email", "E-Mail");
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

        let mut es = HashMap::new();
        es.insert("setup_title", "Configuración Inicial");
        es.insert("setup_welcome", "Bienvenido a AgroCore. Vamos a configurar su sistema.");
        es.insert("admin_password", "Contraseña de Administrador");
        es.insert("tenant_name", "Nombre de la Organización");
        es.insert("tenant_slug", "Identificador (ej. mi-granja)");
        es.insert("finish_setup", "Completar Configuración");
        es.insert("dashboard", "Panel de Control");
        es.insert("sites", "Sitios");
        es.insert("tasks", "Tareas");
        es.insert("users", "Usuarios");
        es.insert("settings", "Ajustes");
        es.insert("first_name", "Nombre");
        es.insert("last_name", "Apellido");
        es.insert("email", "Correo electrónico");

        let mut fr = HashMap::new();
        fr.insert("setup_title", "Configuration Initiale");
        fr.insert("setup_welcome", "Bienvenue sur AgroCore. Configurer votre système.");
        fr.insert("admin_password", "Mot de passe Administrateur");
        fr.insert("tenant_name", "Nom de l'organisation");
        fr.insert("tenant_slug", "Identifiant (ex. ma-ferme)");
        fr.insert("finish_setup", "Terminer la configuration");
        fr.insert("dashboard", "Tableau de bord");
        fr.insert("sites", "Sites");
        fr.insert("tasks", "Tâches");
        fr.insert("users", "Utilisateurs");
        fr.insert("settings", "Paramètres");
        fr.insert("first_name", "Prénom");
        fr.insert("last_name", "Nom");
        fr.insert("email", "E-mail");

        let mut pt = HashMap::new();
        pt.insert("setup_title", "Configuração Inicial");
        pt.insert("setup_welcome", "Bem-vindo ao AgroCore. Vamos configurar o seu sistema.");
        pt.insert("admin_password", "Senha do Administrador");
        pt.insert("tenant_name", "Nome da Organização");
        pt.insert("tenant_slug", "Identificador (ex: minha-fazenda)");
        pt.insert("finish_setup", "Concluir Configuração");
        pt.insert("dashboard", "Painel");
        pt.insert("sites", "Locais");
        pt.insert("tasks", "Tarefas");
        pt.insert("users", "Usuários");
        pt.insert("settings", "Configurações");
        pt.insert("first_name", "Nome");
        pt.insert("last_name", "Sobrenome");
        pt.insert("email", "E-mail");

        t.insert("en", en);
        t.insert("de", de);
        t.insert("es", es);
        t.insert("fr", fr);
        t.insert("pt", pt);

        Self { translations: t }
    }

    pub fn t(&self, lang: &str, key: &str) -> String {
        self.translations.get(lang)
            .and_then(|m| m.get(key))
            .map(|s| s.to_string())
            .unwrap_or_else(|| key.to_string())
    }
}
