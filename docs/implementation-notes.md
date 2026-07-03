# Implementation Notes

This document tracks the current domain shape and open product work for AgroCore.

## Current Direction

- General agricultural management system with sites, tasks, weather, finance, livestock, compliance, and resources.
- Modular Rust workspace with backend, domain, infrastructure, and admin UI crates.
- Focus on data-driven workflows, shared entities, and explicit API contracts.

## Domain Focus

- Sites and polygons for field boundaries.
- Orders and task workflows.
- Compliance and record keeping.
- Weather and phenology.
- Finance and resource tracking.
- Livestock and grazing management.

## Open Technical Work

- Replace remaining static filler values with live data sources.
- Keep the admin UI driven by persisted data rather than demo defaults.
- Prefer generic terminology in new code and documentation.

