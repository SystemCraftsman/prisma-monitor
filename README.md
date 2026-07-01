# Prisma Monitor

Web-based therapy data dashboard for [Löwenstein Prisma](https://www.loewensteinmedical.com/) CPAP/BiPAP devices (Prisma 20A, 25S, SMART, SMART Max).

Parses `.pdat` therapy archives exported from the device's SD card and displays per-night therapy metrics including duration, AHI, apnea/hypopnea breakdown, snoring, and event timelines.

Validated against official Prisma PDF reports with **~0 min average duration error** and **AHI within ±1** across 11 reference nights.

## Features

- Upload and parse `.pdat` files (ZIP archives with event XMLs)
- Per-night therapy duration, AHI, and severity classification
- Apnea/hypopnea subtype breakdown (obstructive, central, leakage, movement)
- Snoring duration tracking
- Trend charts (AHI, therapy hours, event counts over time)
- Device configuration viewer
- Responsive dashboard with Chart.js visualizations

## Screenshots

*Coming soon*

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (2024 edition)
- A `.pdat` file from your Prisma device's SD card

### Build & Run

```bash
cargo run
```

Open http://localhost:3000 and upload your `.pdat` file.

### Export .pdat from your device

1. Insert the SD card from your Prisma device into your computer
2. Copy the `therapy.pdat` file
3. Upload it through the web interface

## Tech Stack

- **Backend:** Rust, Axum 0.8, Tokio
- **Templates:** Askama 0.13
- **Frontend:** HTMX, Chart.js, vanilla CSS
- **Parsing:** quick-xml, zip

## Data Format Documentation

See [`docs/PRISMA_DATA_FORMAT.md`](docs/PRISMA_DATA_FORMAT.md) for comprehensive reverse-engineered documentation of the Prisma `.pdat` file format, including:

- Archive structure and file layout
- Time unit convention (deciseconds)
- RespEventID reference table
- Session lifecycle
- Therapy duration and AHI computation methods

Based on findings from [semyonf/Lowenstein-Prisma-Viewer](https://github.com/semyonf/Lowenstein-Prisma-Viewer) and our own validation against official Prisma reports.

## Privacy

This application runs entirely locally. No data is sent to any external server. Your therapy data stays on your machine.

## License

MIT
