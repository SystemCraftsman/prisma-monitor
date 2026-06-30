# Löwenstein Prisma Data Format Reference

Reverse-engineered format documentation for Löwenstein (formerly Weinmann) Prisma series
CPAP/BiPAP/ASV therapy data archives.

Based on findings from [semyonf/Lowenstein-Prisma-Viewer](https://github.com/semyonf/Lowenstein-Prisma-Viewer)
and our own analysis against official Prisma PDF reports (validated across 11 nights with 0 min average duration error).

Applicable devices: Prisma 20A, 25S, SMART, SMART Max, and other Prisma series.

---

## Archive Files

| File | Extension | Contents |
|------|-----------|----------|
| Configuration archive | `.pcfg` | Device identity and prescribed/user settings |
| Therapy data archive | `.pdat` | Therapy sessions, signals, statistics, logs |

Both files are **standard ZIP archives** (PK header, deflate compression) with custom extensions.

## Archive Structure (therapy.pdat)

```
mnt/flash/
  conf/
    device.xml                          # Device hardware/firmware identification
    configuration.xml                   # Therapy settings (if present)
    config_notifications.ini            # Reminder schedules
    config_operatingtime.ini            # Boot/therapy counters
  data/
    therapy/
      events/
        YYYYMMDD/
          event_NNNNNN.xml              # Per-session therapy events
      signals/
        YYYYMMDD/
          signal_NNNNNN.wmedf           # Per-session waveform data (EDF)
      trendcurves/
        YYYYMMDD/
          trendCurves.tc                # Nightly trend summary
    statistics/
      statistics_year.bin               # Annual statistics (XML format despite .bin)
    debug/
      therapy-sw.log                    # Therapy software log
```

### Date Directory Convention

The device uses a **noon-to-noon** day boundary. Therapy sessions starting before noon
are attributed to the previous calendar day. For example, a session at 01:00 on Jun 5
belongs to the Jun 4 directory. Each date directory = one "night" in Prisma's reporting.

### Event File Pairing

Each therapy session produces a matching pair: `event_NNNNNN.xml` and `signal_NNNNNN.wmedf`
with the same sequence number. Multiple files per date directory represent **parallel
recording channels** of the same therapy session, not separate sessions.

---

## Time Units

> **CRITICAL: EndTime and Duration are in deciseconds (1/10th of a second).**

Divide by 10 to get actual seconds. This was confirmed by cross-referencing epoch
percentages with device display values, and by checking that apnea durations fall in
the clinically expected 10-30 second range.

Each event file has its **own time base starting from 0**. EndTime values are relative
to the start of that session file, NOT a global clock. Events from different files
CANNOT be compared by EndTime.

```
actual_seconds = EndTime / 10
event_start_seconds = (EndTime - Duration) / 10
```

---

## Therapy Events (event_NNNNNN.xml)

```xml
<?xml version="1.0" encoding="utf-8"?>
<desc>
  <DeviceEvent DeviceEventID="0" Time="0" ParameterID="1003" NewValue="2"/>
  <RespEvent RespEventID="101" EndTime="15440" Duration="124" Pressure="0" Strength="5"/>
  ...
</desc>
```

### RespEvent Attributes

| Attribute | Description |
|-----------|-------------|
| `RespEventID` | Event type identifier (see tables below) |
| `EndTime` | **Deciseconds** from session start when the event ended |
| `Duration` | Event duration in **deciseconds** |
| `Pressure` | Therapy pressure at event time (Pa, divide by 100 for cmH2O) |
| `Strength` | Event severity/intensity (scale varies by event type) |
| `Visible` | Optional; `"0"` = hidden/internal event |

### DeviceEvent Attributes

| Attribute | Description |
|-----------|-------------|
| `DeviceEventID` | `0` = config snapshot at session start, `1` = runtime state change |
| `Time` | Seconds from session start |
| `ParameterID` | Parameter identifier (see Configuration section) |
| `NewValue` | New parameter value |

---

## RespEventID Reference

### Epoch Events (2-minute rolling assessments)

Duration = total time spent in that state during the epoch window (deciseconds).

| ID | Name | Notes |
|----|------|-------|
| 1 | Epoch: Severe Obstruction | `% = round(Duration/10 / therapy_seconds * 100)` |
| 2 | Epoch: Mild Obstruction | Same formula |
| 3 | Epoch: Flow Limitation | Same formula |
| 4 | Epoch: Snore | Same formula |
| 5 | Epoch: Periodic Breathing | Same formula |
| 261 | Epoch: Deep Sleep | Matches device "Deep Sleep %" display |

### Clinical Respiratory Events

| ID | Name | Duration | Strength |
|----|------|----------|----------|
| 101 | Obstructive Apnea (OA) | Event duration (ds) | Severity |
| 102 | Central Apnea (CA) | Event duration (ds) | Severity |
| 103 | Apnea (Leakage) | Event duration (ds) | — |
| 105 | Apnea (High Pressure) | Event duration (ds) | — |
| 106 | Apnea (Movement) | Event duration (ds) | — |
| 111 | Obstructive Hypopnea (OH) | Event duration (ds) | Severity |
| 112 | Central Hypopnea (CH) | Event duration (ds) | Severity |
| 113 | Hypopnea (Leakage) | Event duration (ds) | — |
| 121 | RERA | Event duration (ds) | — |
| 131 | Snore | Episode duration (ds) | Intensity |
| 141 | Artifact | Duration (ds) | — |
| 151 | Flow Limitation | Duration (ds) | — |
| 161 | Critical Leakage | Duration (ds) | — |
| 171 | Periodic Breathing | Duration (ds) | — |
| 181 | Cheyne-Stokes Respiration | Duration (ds) | — |
| 221 | Timed Breath | Duration (ds) | — |

### Session/Structure Markers

| ID | Name | Notes |
|----|------|-------|
| 231 | Session Duration | Duration field = session duration marker (ds) |
| 241 | Session End Marker | Paired with 231 |
| 262 | Pressure Change | Pressure titration event |
| 263 | Session boundary | Appears at file end |
| 306 | Mask Off | `Visible="0"` — internal tracking |
| 307 | Mask On | `Visible="0"` — internal tracking |
| 330 | Large Leak | Excessive mask leak detected |
| 1262 | Therapy Pressure Change | Pressure adjustment marker |

### Per-Session Summary Flags (1230-1238)

These appear at session boundaries. The Strength field contains **status flags** (0 or 1),
NOT usable index values. Do not use these for AHI computation.

| ID | Name |
|----|------|
| 1230 | Summary: AHI flag |
| 1231 | Summary: AI flag |
| 1232 | Summary: HI flag |
| 1233 | Summary: Leak95 flag |
| 1234 | Summary: flag |
| 1237 | Summary: PrMed flag |
| 1238 | Summary: PrP95 flag |

---

## Session Lifecycle

A typical event file follows this sequence:

1. **DeviceEventID=0** — Configuration snapshot (all params at Time=0)
2. **DeviceEventID=1, ParameterID=257** — Therapy starts (NewValue=0)
3. **DeviceEventID=1, ParameterID=271** — Phase: 0=ramp, 3=standby, 2=active therapy
4. **RespEventID=1230-1238** — Session summary flags (baseline)
5. **Epoch events (1-5, 261)** — Rolling 2-minute assessments
6. **Clinical events** — Apneas, hypopneas, RERA, snoring, etc.
7. **RespEventID=262/1262** — Pressure changes
8. **RespEventID=231** — Session duration marker
9. **RespEventID=1230-1238** — Final summary flags
10. **RespEventID=241** — Session end marker

---

## Computing Therapy Duration

**Correct method:** Sum the max EndTime (÷10) of each event file per date directory.

```
therapy_seconds = Σ (max_EndTime_per_file / 10)
```

Each file's max EndTime represents its recording duration. Since files are parallel
channels of the same session, they share overlapping time ranges — but each file is
a separate recording segment with its own time base.

**Validated:** This method produces therapy durations matching the official Prisma PDF
report within ±2 minutes across 7 reference nights.

### What NOT to use for therapy duration

| Approach | Problem |
|----------|---------|
| Epoch event count (ID=1,2) × 1200 | Epochs are obstruction assessments, not timers |
| Clinical event gap-split | Misses quiet sleep periods (no events during deep sleep) |
| Snoring event span | Snoring may fire with mask off (microphone-based) |
| Heartbeat/periodic span | Fires regardless of mask status in autoSTART mode |

---

## Computing Clinical Indices

```
therapy_hours = therapy_seconds / 3600

AHI = round((apnea_count + hypopnea_count) / therapy_hours)

  where apnea_count   = count of events 101, 102, 103, 105, 106
        hypopnea_count = count of events 111, 112, 113

AIcent  = round(count_of_102 / therapy_hours)    # Central Apnea Index
HIcent  = round(count_of_112 / therapy_hours)    # Central Hypopnea Index
RERA/h  = round(count_of_121 / therapy_hours)    # RERA Index

Deep Sleep %  = round(epoch_261_duration_ds / 10 / therapy_seconds * 100)
Snore %       = round(epoch_4_duration_ds   / 10 / therapy_seconds * 100)
```

> **Note:** The exact AHIobs formula used by the device remains unsolved. The device
> likely uses time-windowed calculations or minimum duration thresholds not captured
> in the exported event data.

---

## Configuration Parameters

### Therapy Parameters (OBL — prescribed by clinician)

| ID | Name | Unit/Notes |
|----|------|------------|
| 1001 | Device Code | Internal product code |
| 1003 | Therapy Mode | 0=CPAP, 1=APAP, 2=APAP+, 3=BiLevel, 4=ASV, 5=iVAPS |
| 1005 | Autostart | 0=off, 1=on |
| 1014 | Ramp Enabled | 0=off, 1=on |
| 1015 | Ramp Time | Minutes |
| 1084 | Autostart Enabled | 0=off, 1=on |
| 1105 | Auto-Off Time | Minutes after mask removal |
| 1138 | Min Pressure | Pa (÷100 for cmH2O) |
| 1139 | Max Pressure | Pa |
| 1199 | Max Therapy Pressure | Pa — absolute ceiling |

### Pressure Convention

All pressure values are stored in **Pascals (Pa)**. To convert to cmH2O:

```
cmH2O = Pa / 100
```

Example: `val="550"` = 5.50 cmH2O, `val="1600"` = 16.00 cmH2O.

---

## References

- [semyonf/Lowenstein-Prisma-Viewer](https://github.com/semyonf/Lowenstein-Prisma-Viewer) — Original reverse engineering and format documentation
- [OSCAR Prisma loader](https://gitlab.com/pholy/OSCAR-code/-/blob/master/oscar/SleepLib/loader_plugins/prisma_loader.h) — Event ID definitions (referenced by semyonf)
- Löwenstein Medical GmbH & Co. KG — Device manufacturer (formerly Weinmann)
