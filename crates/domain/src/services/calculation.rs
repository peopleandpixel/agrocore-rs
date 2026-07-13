use crate::entities::plant_protection::PlantProtectionAreaMethod;
use chrono::{DateTime, Utc};

pub struct CalculationService;

#[derive(Debug, Clone)]
pub struct MaterialAmountRequest {
    pub method: PlantProtectionAreaMethod,
    pub net_area: f64,
    pub gross_area: Option<f64>,
    pub lane_width: Option<f64>,
    pub total_strike_length: Option<f64>,
    pub is_steep: bool,
    pub dosage_per_ha: f64,
    pub application_date: DateTime<Utc>,
}

impl CalculationService {
    /// Berechnet die benötigte Materialmenge basierend auf der Fläche und der Dosierung.
    /// Berücksichtigt verschiedene Flächenberechnungsmethoden und Steillagenfaktoren.
    pub fn calculate_material_amount(input: MaterialAmountRequest) -> f64 {
        let treated_area = input.method.calculate_treated_area(
            input.net_area,
            input.gross_area,
            input.lane_width,
            input.total_strike_length,
            input.is_steep,
            input.application_date,
        );

        treated_area * input.dosage_per_ha
    }

    /// Berechnet die Wasserabgabe (L/ha) basierend auf der Fahrgeschwindigkeit,
    /// dem Düsendurchfluss und der Reihenbreite.
    pub fn calculate_water_rate(
        speed_kmh: f64,
        nozzle_flow_lmin: f64,
        lane_width: f64,
        number_of_nozzles: u32,
    ) -> f64 {
        if speed_kmh <= 0.0 || lane_width <= 0.0 {
            return 0.0;
        }

        // Formel: (Gesamtdurchfluss L/min * 600) / (Geschwindigkeit km/h * Reihenbreite m)
        let total_flow = nozzle_flow_lmin * number_of_nozzles as f64;
        (total_flow * 600.0) / (speed_kmh * lane_width)
    }

    /// Berechnet das Baumkronen-Volumen (m³/ha) für Spezialkulturen (Oliven, Kork).
    /// Formel: (Kronendurchmesser² * PI / 4) * Baumhöhe * Bäume pro Hektar
    pub fn calculate_tree_crown_volume(
        crown_diameter: f64,
        tree_height: f64,
        trees_per_ha: u32,
    ) -> f64 {
        if crown_diameter <= 0.0 || tree_height <= 0.0 {
            return 0.0;
        }

        let crown_area = (crown_diameter * crown_diameter * std::f64::consts::PI) / 4.0;
        crown_area * tree_height * trees_per_ha as f64
    }

    /// Berechnet den täglichen Futterbedarf für Vieh (in kg Trockenmasse).
    /// Einfache Formel basierend auf dem Körpergewicht und einem Prozentfaktor.
    pub fn calculate_forage_demand(
        body_weight_kg: f64,
        demand_percent: f64,
        animal_count: u32,
    ) -> f64 {
        if body_weight_kg <= 0.0 || demand_percent <= 0.0 {
            return 0.0;
        }

        (body_weight_kg * (demand_percent / 100.0)) * animal_count as f64
    }

    /// Berechnet den Stickstoffbedarf (N) für eine Fläche (kg N).
    /// Basisformel: Fläche (ha) * Bedarf pro ha (kg N/ha)
    pub fn calculate_nitrogen_demand(area_ha: f64, demand_per_ha: f64) -> f64 {
        if area_ha <= 0.0 || demand_per_ha <= 0.0 {
            return 0.0;
        }
        area_ha * demand_per_ha
    }

    /// Berechnet die Erschwernis-Zulage basierend auf Neigung (is_steep)
    /// und anderen Faktoren (z.B. schwerer Boden, Enge).
    pub fn calculate_difficulty_surcharge(
        base_rate: f64,
        is_steep: bool,
        is_heavy_soil: bool,
        is_narrow: bool,
    ) -> f64 {
        let mut multiplier = 1.0;
        if is_steep {
            multiplier += 0.3;
        } // 30% Zuschlag für Steilhang
        if is_heavy_soil {
            multiplier += 0.15;
        } // 15% für schweren Boden
        if is_narrow {
            multiplier += 0.1;
        } // 10% für Enge

        base_rate * multiplier
    }

    /// Schätzt den Erntezeitpunkt basierend auf dem aktuellen BBCH-Stadium,
    /// der Ziel-BBCH (z.B. 89 für Vollreife) und der Durchschnittstemperatur.
    /// Sehr vereinfachtes GDD-Modell (Growing Degree Days).
    pub fn estimate_harvest_date(
        current_bbch: u32,
        target_bbch: u32,
        avg_temp: f64,
        base_temp: f64,
    ) -> Option<u32> {
        if current_bbch >= target_bbch {
            return Some(0); // Bereits reif
        }

        let daily_gdd = (avg_temp - base_temp).max(0.0);
        if daily_gdd <= 0.0 {
            return None; // Kein Wachstum möglich
        }

        // Annahme: Pro BBCH-Punkt werden ca. 15 GDD benötigt (stark vereinfacht)
        let remaining_points = target_bbch - current_bbch;
        let days_needed = (remaining_points as f64 * 15.0) / daily_gdd;

        Some(days_needed.ceil() as u32)
    }

    /// Berechnet die Wirtschaftlichkeit (Profit) pro Hektar.
    /// Formel: (Ertrag * Preis) - (Materialkosten + Arbeitskosten + Maschinenkosten)
    pub fn calculate_profitability(
        yield_amount: f64,
        price_per_unit: f64,
        material_costs: f64,
        labor_costs: f64,
        machinery_costs: f64,
        area_ha: f64,
    ) -> f64 {
        if area_ha <= 0.0 {
            return 0.0;
        }

        let revenue = yield_amount * price_per_unit;
        let total_costs = material_costs + labor_costs + machinery_costs;

        (revenue - total_costs) / area_ha
    }
}
