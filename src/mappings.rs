use std::collections::HashSet;

pub fn resp_event_name(id: u32) -> &'static str {
    match id {
        1 => "eSO",
        2 => "eMO",
        3 => "eFL",
        4 => "eS",
        5 => "eCS",
        6 => "Humidifier is empty",
        101 => "Obstructive Apnea (oA)",
        102 => "Central Apnea (cA)",
        103 => "Apnea leakage",
        104 => "uA_Softstart",
        105 => "Apnea high pressure",
        106 => "Apnea movement",
        108 => "hPr",
        111 => "Obstructive Hypopnea (oH)",
        112 => "Central Hypopnea (cH)",
        113 => "Hypopnea leakage",
        114 => "uH_Softstart",
        115 => "uH_HighPressure",
        116 => "uH_PosChange",
        121 => "RERA",
        131 => "Snore",
        141 => "Artefact",
        151 => "Flattening",
        161 => "Critical leakage",
        171 => "Disconnection",
        172 => "Mask test",
        181 => "CS respiration",
        191 => "Glottal closure",
        211 => "IPAP not reached",
        221 => "Timed breath",
        231 => "Init phase",
        241 => "softSTART",
        242 => "softSTOP",
        251 => "Desaturation",
        252 => "Hypoxemia",
        254 => "Artefact(SpO2)",
        263 => "Unknown(263)",
        301 => "Leakage alert",
        303 => "Disconn. alert",
        305 => "Leakage alert",
        307 => "Disconnection alert",
        309 => "MV low alert",
        311 => "Apnea alert",
        313 => "VT low alert",
        315 => "Volume low alert",
        316 => "Volume high alert",
        317 => "Frequency low alert",
        318 => "Frequency high alert",
        320 => "MV high alert",
        321 => "Pressure low alert",
        322 => "Pressure high alert",
        330 => "Unknown(330)",
        1001 => "DEBUG_EPOCH_SLEEP_ONSET",
        1002 => "DEBUG_EPOCH_SOFTSTART",
        1003 => "DEBUG_EPOCH_TOOSHORT",
        1004 => "DEBUG_EPOCH_FORCEDEND",
        1005 => "DEBUG_EPOCH_LEAK",
        1006 => "DEBUG_EPOCH_UNRELIABLE",
        1007 => "DEBUG_EPOCH_CENTRAL",
        1008 => "DEBUG_EPOCH_SEVERE_EVENT",
        1101 => "DEBUG_APNEA_TOOSHORT",
        1102 => "DEBUG_APNEA_TOOLONG",
        1103 => "DEBUG_APNEA_UNSPECIFIED",
        1104 => "DEBUG_APNEA_MIXED",
        1105 => "DEBUG_APNEA_FORCEDEND",
        1106 => "DEBUG_INVALID_APNEA_TOOSHORT",
        1107 => "DEBUG_APNEA_COMBINED",
        1111 => "DEBUG_HYPOPNEA_TOOSHORT",
        1112 => "DEBUG_HYPOPNEA_TOOLONG",
        1113 => "DEBUG_HYPOPNEA_OBSTRUCTIVE_FLAT",
        1114 => "DEBUG_HYPOPNEA_INVALID_APNEA",
        1115 => "DEBUG_HYPOPNEA_UNSPECIFIED",
        1116 => "DEBUG_HYPOPNEA_MIXED",
        1117 => "DEBUG_HYPOPNEA_FORCEDEND",
        1118 => "DEBUG_INVALID_HYPOPNEA_TOOSHORT",
        1120 => "DEBUG_RERA_TOO_SHORT",
        1121 => "DEBUG_RERA_INVALID_LEAK",
        1122 => "DEBUG_RERA_INVALID_APNEA",
        1123 => "DEBUG_RERA_INVALID_HYPOPNEA",
        1124 => "DEBUG_RERA_INVALID_DELTA_PDIFF",
        1125 => "DEBUG_RERA_INVALID_DELTA_LEAK",
        1126 => "DEBUG_RERA_MILD",
        1127 => "DEBUG_RERA_MILDSNORE",
        1128 => "DEBUG_RERA_MILDFLAT",
        1129 => "DEBUG_RERA_SEVERE",
        1130 => "DEBUG_RERA_FORCEDEND",
        1131 => "DEBUG_SNORE_INVALID_LEAK",
        1132 => "DEBUG_SNORE_INVALID_HIGHPRESSURE",
        1133 => "DEBUG_SNORE_INVALID_TOOSHORT",
        1134 => "DEBUG_SNORE_FORCEDEND",
        1141 => "DEBUG_ARTEFACT_TOOSHORT",
        1151 => "DEBUG_FLATTENING_INVALID_LEAK",
        1152 => "DEBUG_FLATTENING_INVALID_TOOSHORT",
        1153 => "DEBUG_FLATTENING_FORCEDEND",
        1154 => "DEBUG_FLATTENING_INVALID",
        1201 => "DEBUG_AASM_TITRATION_CYCLE",
        1221 => "DEBUG_TIMED_BREATH",
        1222 => "DEBUG_TIMED_BREATH_OBSTRUCTIVE",
        1223 => "DEBUG_TIMED_BREATH_CENTRAL",
        1224 => "DEBUG_TIMED_BREATH_INVALID_TOOSHORT",
        1230 => "SESSION_START_MARKER",
        1231 => "SESSION_END_MARKER",
        1232 => "SESSION_MARKER_1232",
        1233 => "SESSION_MARKER_1233",
        1234 => "SESSION_MARKER_1234",
        1235 => "SESSION_MARKER_1235",
        1237 => "SESSION_MARKER_1237",
        1238 => "SESSION_MARKER_1238",
        1240 => "SESSION_MARKER_1240",
        1241 => "SESSION_MARKER_1241",
        _ => "Unknown",
    }
}

pub fn param_name(id: u32) -> &'static str {
    match id {
        1001 => "ModulVersion",
        1002 => "Device",
        1003 => "Mode",
        1011 => "TI",
        1012 => "TE",
        1014 => "RampEx",
        1015 => "WmTrackEx",
        1016 => "TargetVolume",
        1017 => "IpapSpeed",
        1020 => "IntraBreathPressCtrl",
        1083 => "HumidifierLevel",
        1084 => "AutoStart",
        1085 => "MaskTestPress",
        1086 => "MaskTestDuration",
        1091 => "TubeType",
        1092 => "BacteriumFilter",
        1123 => "SoftPapLevel",
        1125 => "SoftStartPress",
        1127 => "SoftStartTime",
        1128 => "SoftStartTimeMax",
        1138 => "EepapMin",
        1139 => "EepapMax",
        1140 => "PdiffNorm",
        1141 => "PdiffMax",
        1150 => "AbsolutPdiffMinTimed",
        1154 => "ExtraObstructionProtection",
        1158 => "RelativeInspirationDuration",
        1160 => "RampIn",
        1162 => "WmTrack",
        1199 => "IPapMax",
        1200 => "IPap",
        1201 => "Epap",
        1202 => "AlarmLeakActive",
        1203 => "AlarmDisconnectionActive",
        1206 => "AlertApneaLevel",
        1207 => "AlertVtLowLevel",
        1208 => "AlertAmvLowLevel",
        1209 => "Apap_dyn",
        1212 => "SoftStopTimeMax",
        1213 => "SoftStopTime",
        1214 => "TiMin",
        1215 => "TiMax",
        1216 => "TiTimed",
        1217 => "HumClimaCtrl",
        1218 => "AutoStop",
        1219 => "AutoPdiffActive",
        1220 => "SoftStartDiffRamp",
        1223 => "TargetFlow",
        1224 => "O2Flow",
        _ => "Unknown",
    }
}

pub fn mode_name(id: u32) -> &'static str {
    match id {
        1 => "CPAP",
        2 => "CPAP-A (AutoCPAP)",
        3 => "S (Spontaneous)",
        4 => "ST (Spontaneous-Timed)",
        5 => "T (Timed)",
        6 => "aS (auto-S)",
        7 => "aST (auto-ST)",
        8 => "aT (auto-T)",
        9 => "APAP (Auto-CPAP)",
        10 => "S/T",
        _ => "Unknown",
    }
}

pub fn ahi_apnea_ids() -> HashSet<u32> {
    [101, 102, 103, 104, 105, 106].into_iter().collect()
}

pub fn ahi_hypopnea_ids() -> HashSet<u32> {
    [111, 112, 113, 114, 115, 116].into_iter().collect()
}

pub fn is_clinical_event(id: u32) -> bool {
    id <= 322
}

pub fn is_debug_event(id: u32) -> bool {
    (1000..1230).contains(&id)
}

pub fn is_session_marker(id: u32) -> bool {
    id >= 1230
}

pub fn ahi_severity(ahi: f64) -> &'static str {
    if ahi < 5.0 {
        "Normal"
    } else if ahi < 15.0 {
        "Mild"
    } else if ahi < 30.0 {
        "Moderate"
    } else {
        "Severe"
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventCategory {
    Clinical,
    Debug,
    SessionMarker,
}

pub fn event_category(id: u32) -> EventCategory {
    if is_session_marker(id) {
        EventCategory::SessionMarker
    } else if is_debug_event(id) {
        EventCategory::Debug
    } else {
        EventCategory::Clinical
    }
}
