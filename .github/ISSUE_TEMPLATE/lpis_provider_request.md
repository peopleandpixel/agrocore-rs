---
name: LPIS Provider Request
about: Request support for a new country's LPIS (Land Parcel Identification System)
title: "[LPIS] Add <Country> <System Name> provider"
labels: ["lpis", "enhancement", "triage"]
assignees: []
---

## Country & System
- **Country**: [e.g., Belgium, Ireland, Czech Republic, Greece, etc.]
- **LPIS Name**: [e.g., LPIS-BE, LPIS-IE, LPIS-CZ, etc.]
- **Official Website**: [URL to government portal]

## Technical Details
| Property | Value |
|----------|-------|
| **WFS Endpoint** | [GetCapabilities URL] |
| **Authentication** | [None / API Key / OAuth2 / IP Allowlist / Client Cert] |
| **Feature Type** | [e.g., `parcels`, `agricultural_parcels`, `fields`] |
| **Geometry Column** | [e.g., `geom`, `geometry`, `wkb_geometry`] |
| **CRS** | [e.g., EPSG:4326, EPSG:3857, EPSG:3035] |
| **Key Field** | [Unique parcel identifier field name] |
| **Area Field** | [Hectares field name, if present] |
| **Crop Code Field** | [Crop/variety field name, if present] |
| **Language** | [Field names / enum values language] |

## Rate Limits & Constraints
- **Max Features per Request**: [e.g., 1000, 5000, unlimited]
- **Pagination Support**: [Yes/No – `startIndex`/`count` or `RESULTTYPE=hits`]
- **CQL Filter Support**: [Yes/No – which operators?]
- **BBOX Filter**: [Yes/No]
- **Request Timeout**: [Observed typical response time]
- **Rate Limit**: [Requests/minute, IP-based, key-based]

## Sample Data
If possible, provide:
- GetCapabilities XML (or link)
- DescribeFeatureType XML (or link)
- 1-2 sample GetFeature responses (GML/GeoJSON)

## Legal / Licensing
- **Data License**: [Open Data / CC-BY / Custom / Unknown]
- **Terms of Use URL**: [Link]
- **Attribution Required**: [Yes/No – text]

## Implementation Readiness
- [ ] I can test against the live endpoint
- [ ] I have access to test credentials (if auth required)
- [ ] I can contribute the provider implementation (Rust)
- [ ] I can help with Admin UI translations for this country
- [ ] I know farmers/users in this country who would test

## Additional Context
- Existing open-source clients for this LPIS (QGIS plugins, scripts, etc.)
- Known issues / quirks with the WFS service
- Contact at the authority (if you have one)