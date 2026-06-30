use crate::entities::plant_protection::PlantProtectionAreaMethod;
use chrono::{DateTime, Utc};

pub struct CalculationService;

impl CalculationService {
    /// Berechnet die benötigte Materialmenge basierend auf der Fläche und der Dosierung.
    /// Berücksichtigt verschiedene Flächenberechnungsmethoden und Steillagenfaktoren.
    pub fn calculate_material_amount(
        method: &PlantProtectionAreaMethod,
        net_area: f64,
        gross_area: Option<f64>,
        lane_width: Option<f64>,
        total_strike_length: Option<f64>,
        is_steep: bool,
        dosage_per_ha: f64,
        application_date: DateTime<Utc>,
    ) -> f64 {
        let treated_area = method.calculate_treated_area(
            net_area,
            gross_area,
            lane_width,
            total_strike_length,
            is_steep,
            application_date,
        );

        treated_area * dosage_per_ha
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
    pub fn calculate_nitrogen_demand(
        area_ha: f64,
        demand_per_ha: f64,
    ) -> f64 {
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
        if is_steep { multiplier += 0.3; } // 30% Zuschlag für Steilhang
        if is_heavy_soil { multiplier += 0.15; } // 15% für schweren Boden
        if is_narrow { multiplier += 0.1; } // 10% für Enge
        
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
        if area_ha <= 0.0 { return 0.0; }
        
        let revenue = yield_amount * price_per_unit;
        let total_costs = material_costs + labor_costs + machinery_costs;
        
        (revenue - total_costs) / area_ha
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_calculate_material_amount() {
        let method = PlantProtectionAreaMethod::NetArea;
        let date = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let amount = CalculationService::calculate_material_amount(
            &method, 1.0, None, None, None, false, 5.0, date
        );
        assert_eq!(amount, 5.0);
    }

    #[test]
    fn test_calculate_water_rate() {
        // Beispiel: 6 km/h, 1.5 L/min pro Düse, 2m Reihenbreite, 10 Düsen
        // (15 * 600) / (6 * 2) = 9000 / 12 = 750 L/ha
        let rate = CalculationService::calculate_water_rate(6.0, 1.5, 2.0, 10);
        assert_eq!(rate, 750.0);
    }

    #[test]
    fn test_calculate_tree_crown_volume() {
        // 3m Durchmesser, 4m Höhe, 400 Bäume/ha
        // Fläche = 3² * PI / 4 = 7.0685...
        // Volumen = 7.0685 * 4 * 400 = 11309.73...
        let vol = CalculationService::calculate_tree_crown_volume(3.0, 4.0, 400);
        assert!(vol > 11309.0 && vol < 11310.0);
    }

    #[test]
    fn test_calculate_forage_demand() {
        // 500kg Kuh, 3% Bedarf, 10 Tiere
        // 500 * 0.03 * 10 = 150kg
        let demand = CalculationService::calculate_forage_demand(500.0, 3.0, 10);
        assert_eq!(demand, 150.0);
    }

    #[test]
    fn test_calculate_nitrogen_demand() {
        // 2.5 ha, 140kg N/ha
        let n_demand = CalculationService::calculate_nitrogen_demand(2.5, 140.0);
        assert_eq!(n_demand, 350.0);
    }

    #[test]
    fn test_calculate_difficulty_surcharge() {
        // Basis 100€, steil + schwerer Boden
        // 100 * (1 + 0.3 + 0.15) = 145
        let price = CalculationService::calculate_difficulty_surcharge(100.0, true, true, false);
        assert_eq!(price, 145.0);
    }

    #[test]
    fn test_estimate_harvest_date() {
        // BBCH 75 -> 89 (14 Punkte). 20 Grad, Basis 10 Grad -> 10 GDD/Tag.
        // 14 * 15 / 10 = 21 Tage.
        let days = CalculationService::estimate_harvest_date(75, 89, 20.0, 10.0);
        assert_eq!(days, Some(21));
    }
}
