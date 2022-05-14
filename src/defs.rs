use simplelog::*;
use std::ops::Add;
use std::collections::HashMap;




pub struct Alarm {
    name: &'static str,
    code: u16,
    severity: &'static str,
}

impl Alarm {
    pub fn new(name: &'static str, code: u16, severity: &'static str) -> Self {
        Self {
            name,
            code,
            severity,
        }
    }
}


pub struct Sun2000State {
    pub device_status: Option<u16>,
    pub storage_status: Option<i16>,
    pub grid_code: Option<u16>,
    pub state_1: Option<u16>,
    pub state_2: Option<u16>,
    pub state_3: Option<u32>,
    pub alarm_1: Option<u16>,
    pub alarm_2: Option<u16>,
    pub alarm_3: Option<u16>,
    pub fault_code: Option<u16>,
}

impl Sun2000State {
    pub fn get_device_status_description(code: u16) -> &'static str {
        match code {
            0x0000 => "Standby: initializing",
            0x0001 => "Standby: detecting insulation resistance",
            0x0002 => "Standby: detecting irradiation",
            0x0003 => "Standby: grid detecting",
            0x0100 => "Starting",
            0x0200 => "On-grid",
            0x0201 => "Grid Connection: power limited",
            0x0202 => "Grid Connection: self-derating",
            0x0300 => "Shutdown: fault",
            0x0301 => "Shutdown: command",
            0x0302 => "Shutdown: OVGR",
            0x0303 => "Shutdown: communication disconnected",
            0x0304 => "Shutdown: power limited",
            0x0305 => "Shutdown: manual startup required",
            0x0306 => "Shutdown: DC switches disconnected",
            0x0307 => "Shutdown: rapid cutoff",
            0x0308 => "Shutdown: input underpowered",
            0x0401 => "Grid scheduling: cosphi-P curve",
            0x0402 => "Grid scheduling: Q-U curve",
            0x0403 => "Grid scheduling: PF-U curve",
            0x0404 => "Grid scheduling: dry contact",
            0x0405 => "Grid scheduling: Q-P curve",
            0x0500 => "Spot-check ready",
            0x0501 => "Spot-checking",
            0x0600 => "Inspecting",
            0x0700 => "AFCI self check",
            0x0800 => "I-V scanning",
            0x0900 => "DC input detection",
            0x0a00 => "Running: off-grid charging",
            0xa000 => "Standby: no irradiation",
            _ => "Unknown State",
        }
    }

    pub fn get_storage_status_description(code: i16) -> &'static str {
        match code {
            0 => "offline",
            1 => "standby",
            2 => "running",
            3 => "fault",
            4 => "sleep mode",
            _ => "Unknown State",
        }
    }

    #[rustfmt::skip]
    pub fn get_grid_code_description(code: u16) -> String {
        let grid_code = match code {
            0 => ("VDE-AR-N-4105", "Germany 🇩🇪"),
            1 => ("NB/T 32004", "China 🇨🇳"),
            2 => ("UTE C 15-712-1(A)", "France 🇫🇷"),
            3 => ("UTE C 15-712-1(B)", "France 🇫🇷"),
            4 => ("UTE C 15-712-1(C)", "France 🇫🇷"),
            5 => ("VDE 0126-1-1-BU", "Bulgary 🇧🇬"),
            6 => ("VDE 0126-1-1-GR(A)", "Greece 🇬🇷"),
            7 => ("VDE 0126-1-1-GR(B)", "Greece 🇬🇷"),
            8 => ("BDEW-MV", "Germany 🇩🇪"),
            9 => ("G59-England", "UK 🇬🇧"),
            10 => ("G59-Scotland", "UK 🇬🇧"),
            11 => ("G83-England", "UK 🇬🇧"),
            12 => ("G83-Scotland", "UK 🇬🇧"),
            13 => ("CEI0-21", "Italy 🇮🇹"),
            14 => ("EN50438-CZ", "Czech Republic 🇨🇿"),
            15 => ("RD1699/661", "Spain 🇪🇸"),
            16 => ("RD1699/661-MV480", "Spain 🇪🇸"),
            17 => ("EN50438-NL", "Netherlands 🇳🇱"),
            18 => ("C10/11", "Belgium 🇧🇪"),
            19 => ("AS4777", "Australia 🇦🇺"),
            20 => ("IEC61727", "General"),
            21 => ("Custom (50 Hz)", "Custom"),
            22 => ("Custom (60 Hz)", "Custom"),
            23 => ("CEI0-16", "Italy 🇮🇹"),
            24 => ("CHINA-MV480", "China 🇨🇳"),
            25 => ("CHINA-MV", "China 🇨🇳"),
            26 => ("TAI-PEA", "Thailand 🇹🇭"),
            27 => ("TAI-MEA", "Thailand 🇹🇭"),
            28 => ("BDEW-MV480", "Germany 🇩🇪"),
            29 => ("Custom MV480 (50 Hz)", "Custom"),
            30 => ("Custom MV480 (60 Hz)", "Custom"),
            31 => ("G59-England-MV480", "UK 🇬🇧"),
            32 => ("IEC61727-MV480", "General"),
            33 => ("UTE C 15-712-1-MV480", "France 🇫🇷"),
            34 => ("TAI-PEA-MV480", "Thailand 🇹🇭"),
            35 => ("TAI-MEA-MV480", "Thailand 🇹🇭"),
            36 => ("EN50438-DK-MV480", "Denmark 🇩🇰"),
            37 => ("Japan standard (50 Hz)", "Japan 🇯🇵"),
            38 => ("Japan standard (60 Hz)", "Japan 🇯🇵"),
            39 => ("EN50438-TR-MV480", "Turkey 🇹🇷"),
            40 => ("EN50438-TR", "Turkey 🇹🇷"),
            41 => ("C11/C10-MV480", "Belgium 🇧🇪"),
            42 => ("Philippines", "Philippines 🇵🇭"),
            43 => ("Philippines-MV480", "Philippines 🇵🇭"),
            44 => ("AS4777-MV480", "Australia 🇦🇺"),
            45 => ("NRS-097-2-1", "South Africa 🇿🇦"),
            46 => ("NRS-097-2-1-MV480", "South Africa 🇿🇦"),
            47 => ("KOREA", "South Korea 🇰🇷"),
            48 => ("IEEE 1547-MV480", "USA 🇺🇸"),
            49 => ("IEC61727-60Hz", "General"),
            50 => ("IEC61727-60Hz-MV480", "General"),
            51 => ("CHINA_MV500", "China 🇨🇳"),
            52 => ("ANRE", "Romania 🇷🇴"),
            53 => ("ANRE-MV480", "Romania 🇷🇴"),
            54 => ("ELECTRIC RULE NO.21-MV480", "California, USA 🇺🇸"),
            55 => ("HECO-MV480", "Hawaii, USA 🇺🇸"),
            56 => ("PRC_024_Eastern-MV480", "Eastern USA 🇺🇸"),
            57 => ("PRC_024_Western-MV480", "Western USA 🇺🇸"),
            58 => ("PRC_024_Quebec-MV480", "Quebec, Canada 🇨🇦"),
            59 => ("PRC_024_ERCOT-MV480", "Texas, USA 🇺🇸"),
            60 => ("PO12.3-MV480", "Spain 🇪🇸"),
            61 => ("EN50438_IE-MV480", "Ireland 🇮🇪"),
            62 => ("EN50438_IE", "Ireland 🇮🇪"),
            63 => ("IEEE 1547a-MV480", "USA 🇺🇸"),
            64 => ("Japan standard (MV420-50 Hz)", "Japan 🇯🇵"),
            65 => ("Japan standard (MV420-60 Hz)", "Japan 🇯🇵"),
            66 => ("Japan standard (MV440-50 Hz)", "Japan 🇯🇵"),
            67 => ("Japan standard (MV440-60 Hz)", "Japan 🇯🇵"),
            68 => ("IEC61727-50Hz-MV500", "General"),
            70 => ("CEI0-16-MV480", "Italy 🇮🇹"),
            71 => ("PO12.3", "Spain 🇪🇸"),
            72 => ("Japan standard (MV400-50 Hz)", "Japan 🇯🇵"),
            73 => ("Japan standard (MV400-60 Hz)", "Japan 🇯🇵"),
            74 => ("CEI0-21-MV480", "Italy 🇮🇹"),
            75 => ("KOREA-MV480", "South Korea 🇰🇷"),
            76 => ("Egypt ETEC", "Egypt 🇪🇬"),
            77 => ("Egypt ETEC-MV480", "Egypt 🇪🇬"),
            78 => ("CHINA_MV800", "China 🇨🇳"),
            79 => ("IEEE 1547-MV600", "USA 🇺🇸"),
            80 => ("ELECTRIC RULE NO.21-MV600", "California, USA 🇺🇸"),
            81 => ("HECO-MV600", "Hawaii, USA 🇺🇸"),
            82 => ("PRC_024_Eastern-MV600", "Eastern USA 🇺🇸"),
            83 => ("PRC_024_Western-MV600", "Western USA 🇺🇸"),
            84 => ("PRC_024_Quebec-MV600", "Quebec, Canada 🇨🇦"),
            85 => ("PRC_024_ERCOT-MV600", "Texas, USA 🇺🇸"),
            86 => ("IEEE 1547a-MV600", "USA 🇺🇸"),
            87 => ("EN50549-LV", "Ireland 🇮🇪"),
            88 => ("EN50549-MV480", "Ireland 🇮🇪"),
            89 => ("Jordan-Transmission", "Jordan 🇯🇴"),
            90 => ("Jordan-Transmission-MV480", "Jordan 🇯🇴"),
            91 => ("NAMIBIA", "Namibia 🇳🇦"),
            92 => ("ABNT NBR 16149", "Brazil 🇧🇷"),
            93 => ("ABNT NBR 16149-MV480", "Brazil 🇧🇷"),
            94 => ("SA_RPPs", "South Africa 🇿🇦"),
            95 => ("SA_RPPs-MV480", "South Africa 🇿🇦"),
            96 => ("INDIA", "India 🇮🇳"),
            97 => ("INDIA-MV500", "India 🇮🇳"),
            98 => ("ZAMBIA", "Zambia 🇿🇲"),
            99 => ("ZAMBIA-MV480", "Zambia 🇿🇲"),
            100 => ("Chile", "Chile 🇨🇱"),
            101 => ("Chile-MV480", "Chile 🇨🇱"),
            102 => ("CHINA-MV500-STD", "China 🇨🇳"),
            103 => ("CHINA-MV480-STD", "China 🇨🇳"),
            104 => ("Mexico-MV480", "Mexico 🇲🇽"),
            105 => ("Malaysian", "Malaysia 🇲🇾"),
            106 => ("Malaysian-MV480", "Malaysia 🇲🇾"),
            107 => ("KENYA_ETHIOPIA", "East Africa"),
            108 => ("KENYA_ETHIOPIA-MV480", "East Africa"),
            109 => ("G59-England-MV800", "UK 🇬🇧"),
            110 => ("NEGERIA", "Negeria 🇳🇬"),
            111 => ("NEGERIA-MV480", "Negeria 🇳🇬"),
            112 => ("DUBAI", "Dubai 🇦🇪"),
            113 => ("DUBAI-MV480", "Dubai 🇦🇪"),
            114 => ("Northern Ireland", "Northern Ireland"),
            115 => ("Northern Ireland-MV480", "Northern Ireland"),
            116 => ("Cameroon", "Cameroon 🇨🇲"),
            117 => ("Cameroon-MV480", "Cameroon 🇨🇲"),
            118 => ("Jordan Distribution", "Jordan 🇯🇴"),
            119 => ("Jordan Distribution-MV480", "Jordan 🇯🇴"),
            120 => ("Custom MV600-50 Hz", "Custom"),
            121 => ("AS4777-MV800", "Australia 🇦🇺"),
            122 => ("INDIA-MV800", "India 🇮🇳"),
            123 => ("IEC61727-MV800", "General"),
            124 => ("BDEW-MV800", "Germany 🇩🇪"),
            125 => ("ABNT NBR 16149-MV800", "Brazil 🇧🇷"),
            126 => ("UTE C 15-712-1-MV800", "France 🇫🇷"),
            127 => ("Chile-MV800", "Chile 🇨🇱"),
            128 => ("Mexico-MV800", "Mexico 🇲🇽"),
            129 => ("EN50438-TR-MV800", "Turkey 🇹🇷"),
            130 => ("TAI-PEA-MV800", "Thailand 🇹🇭"),
            133 => ("NRS-097-2-1-MV800", "South Africa 🇿🇦"),
            134 => ("SA_RPPs-MV800", "South Africa 🇿🇦"),
            135 => ("Jordan-Transmission-MV800", "Jordan 🇯🇴"),
            136 => ("Jordan-Distribution-MV800", "Jordan 🇯🇴"),
            137 => ("Egypt ETEC-MV800", "Egypt 🇪🇬"),
            138 => ("DUBAI-MV800", "Dubai 🇦🇪"),
            139 => ("SAUDI-MV800", "Saudi Arabia 🇸🇦"),
            140 => ("EN50438_IE-MV800", "Ireland 🇮🇪"),
            141 => ("EN50549-MV800", "Ireland 🇮🇪"),
            142 => ("Northern Ireland-MV800", "Northern Ireland"),
            143 => ("CEI0-21-MV800", "Italy 🇮🇹"),
            144 => ("IEC 61727-MV800-60Hz", "General"),
            145 => ("NAMIBIA_MV480", "Namibia 🇳🇦"),
            146 => ("Japan (LV202-50 Hz)", "Japan 🇯🇵"),
            147 => ("Japan (LV202-60 Hz)", "Japan 🇯🇵"),
            148 => ("Pakistan-MV800", "Pakistan 🇵🇰"),
            149 => ("BRASIL-ANEEL-MV800", "Brazil 🇧🇷"),
            150 => ("Israel-MV800", "Israel 🇮🇱"),
            151 => ("CEI0-16-MV800", "Italy 🇮🇹"),
            152 => ("ZAMBIA-MV800", "Zambia 🇿🇲"),
            153 => ("KENYA_ETHIOPIA-MV800", "East Africa"),
            154 => ("NAMIBIA_MV800", "Namibia 🇳🇦"),
            155 => ("Cameroon-MV800", "Cameroon 🇨🇲"),
            156 => ("NIGERIA-MV800", "Nigeria 🇳🇬"),
            157 => ("ABUDHABI-MV800", "Abu Dhabi 🇦🇪"),
            158 => ("LEBANON", "Lebanon 🇱🇧"),
            159 => ("LEBANON-MV480", "Lebanon 🇱🇧"),
            160 => ("LEBANON-MV800", "Lebanon 🇱🇧"),
            161 => ("ARGENTINA-MV800", "Argentina 🇦🇷"),
            162 => ("ARGENTINA-MV500", "Argentina 🇦🇷"),
            163 => ("Jordan-Transmission-HV", "Jordan 🇯🇴"),
            164 => ("Jordan-Transmission-HV480", "Jordan 🇯🇴"),
            165 => ("Jordan-Transmission-HV800", "Jordan 🇯🇴"),
            166 => ("TUNISIA", "Tunisia 🇹🇳"),
            167 => ("TUNISIA-MV480", "Tunisia 🇹🇳"),
            168 => ("TUNISIA-MV800", "Tunisia 🇹🇳"),
            169 => ("JAMAICA-MV800", "Jamaica 🇯🇲"),
            170 => ("AUSTRALIA-NER", "Australia 🇦🇺"),
            171 => ("AUSTRALIA-NER-MV480", "Australia 🇦🇺"),
            172 => ("AUSTRALIA-NER-MV800", "Australia 🇦🇺"),
            173 => ("SAUDI", "Saudi Arabia 🇸🇦"),
            174 => ("SAUDI-MV480", "Saudi Arabia 🇸🇦"),
            175 => ("Ghana-MV480", "Ghana 🇬🇭"),
            176 => ("Israel", "Israel 🇮🇱"),
            177 => ("Israel-MV480", "Israel 🇮🇱"),
            178 => ("Chile-PMGD", "Chile 🇨🇱"),
            179 => ("Chile-PMGD-MV480", "Chile 🇨🇱"),
            180 => ("VDE-AR-N4120-HV", "Germany 🇩🇪"),
            181 => ("VDE-AR-N4120-HV480", "Germany 🇩🇪"),
            182 => ("VDE-AR-N4120-HV800", "Germany 🇩🇪"),
            183 => ("IEEE 1547-MV800", "USA 🇺🇸"),
            184 => ("Nicaragua-MV800", "Nicaragua 🇳🇮"),
            185 => ("IEEE 1547a-MV800", "USA 🇺🇸"),
            186 => ("ELECTRIC RULE NO.21-MV800", "California, USA 🇺🇸"),
            187 => ("HECO-MV800", "Hawaii, USA 🇺🇸"),
            188 => ("PRC_024_Eastern-MV800", "Eastern USA 🇺🇸"),
            189 => ("PRC_024_Western-MV800", "Western USA 🇺🇸"),
            190 => ("PRC_024_Quebec-MV800", "Quebec, Canada 🇨🇦"),
            191 => ("PRC_024_ERCOT-MV800", "Texas, USA 🇺🇸"),
            192 => ("Custom-MV800-50Hz", "Custom"),
            193 => ("RD1699/661-MV800", "Spain 🇪🇸"),
            194 => ("PO12.3-MV800", "Spain 🇪🇸"),
            195 => ("Mexico-MV600", "Mexico 🇲🇽"),
            196 => ("Vietnam-MV800", "Vietnam 🇻🇳"),
            197 => ("CHINA-LV220/380", "China 🇨🇳"),
            198 => ("SVG-LV", "Dedicated"),
            199 => ("Vietnam", "Vietnam 🇻🇳"),
            200 => ("Vietnam-MV480", "Vietnam 🇻🇳"),
            201 => ("Chile-PMGD-MV800", "Chile 🇨🇱"),
            202 => ("Ghana-MV800", "Ghana 🇬🇭"),
            203 => ("TAIPOWER", "Taiwan 🇹🇼"),
            204 => ("TAIPOWER-MV480", "Taiwan 🇹🇼"),
            205 => ("TAIPOWER-MV800", "Taiwan 🇹🇼"),
            206 => ("IEEE 1547-LV208", "USA 🇺🇸"),
            207 => ("IEEE 1547-LV240", "USA 🇺🇸"),
            208 => ("IEEE 1547a-LV208", "USA 🇺🇸"),
            209 => ("IEEE 1547a-LV240", "USA 🇺🇸"),
            210 => ("ELECTRIC RULE NO.21-LV208", "USA 🇺🇸"),
            211 => ("ELECTRIC RULE NO.21-LV240", "USA 🇺🇸"),
            212 => ("HECO-O+M+H-LV208", "USA 🇺🇸"),
            213 => ("HECO-O+M+H-LV240", "USA 🇺🇸"),
            214 => ("PRC_024_Eastern-LV208", "USA 🇺🇸"),
            215 => ("PRC_024_Eastern-LV240", "USA 🇺🇸"),
            216 => ("PRC_024_Western-LV208", "USA 🇺🇸"),
            217 => ("PRC_024_Western-LV240", "USA 🇺🇸"),
            218 => ("PRC_024_ERCOT-LV208", "USA 🇺🇸"),
            219 => ("PRC_024_ERCOT-LV240", "USA 🇺🇸"),
            220 => ("PRC_024_Quebec-LV208", "USA 🇺🇸"),
            221 => ("PRC_024_Quebec-LV240", "USA 🇺🇸"),
            222 => ("ARGENTINA-MV480", "Argentina 🇦🇷"),
            223 => ("Oman", "Oman 🇴🇲"),
            224 => ("Oman-MV480", "Oman 🇴🇲"),
            225 => ("Oman-MV800", "Oman 🇴🇲"),
            226 => ("Kuwait", "Kuwait 🇰🇼"),
            227 => ("Kuwait-MV480", "Kuwait 🇰🇼"),
            228 => ("Kuwait-MV800", "Kuwait 🇰🇼"),
            229 => ("Bangladesh", "Bangladesh 🇧🇩"),
            230 => ("Bangladesh-MV480", "Bangladesh 🇧🇩"),
            231 => ("Bangladesh-MV800", "Bangladesh 🇧🇩"),
            232 => ("Chile-Net_Billing", "Chile 🇨🇱"),
            233 => ("EN50438-NL-MV480", "Netherlands 🇳🇱"),
            234 => ("Bahrain", "Bahrain 🇧🇭"),
            235 => ("Bahrain-MV480", "Bahrain 🇧🇭"),
            236 => ("Bahrain-MV800", "Bahrain 🇧🇭"),
            238 => ("Japan-MV550-50Hz", "Japan 🇯🇵"),
            239 => ("Japan-MV550-60Hz", "Japan 🇯🇵"),
            241 => ("ARGENTINA", "Argentina 🇦🇷"),
            242 => ("KAZAKHSTAN-MV800", "Kazakhstan 🇰🇿"),
            243 => ("Mauritius", "Mauritius 🇲🇺"),
            244 => ("Mauritius-MV480", "Mauritius 🇲🇺"),
            245 => ("Mauritius-MV800", "Mauritius 🇲🇺"),
            246 => ("Oman-PDO-MV800", "Oman 🇴🇲"),
            247 => ("EN50438-SE", "Sweden 🇸🇪"),
            248 => ("TAI-MEA-MV800", "Thailand 🇹🇭"),
            249 => ("Pakistan", "Pakistan 🇵🇰"),
            250 => ("Pakistan-MV480", "Pakistan 🇵🇰"),
            251 => ("PORTUGAL-MV800", "Portugal 🇵🇹"),
            252 => ("HECO-L+M-LV208", "USA 🇺🇸"),
            253 => ("HECO-L+M-LV240", "USA 🇺🇸"),
            254 => ("C10/11-MV800", "Belgium 🇧🇪"),
            255 => ("Austria", "Austria 🇦🇹"),
            256 => ("Austria-MV480", "Austria 🇦🇹"),
            257 => ("G98", "UK 🇬🇧"),
            258 => ("G99-TYPEA-LV", "UK 🇬🇧"),
            259 => ("G99-TYPEB-LV", "UK 🇬🇧"),
            260 => ("G99-TYPEB-HV", "UK 🇬🇧"),
            261 => ("G99-TYPEB-HV-MV480", "UK 🇬🇧"),
            262 => ("G99-TYPEB-HV-MV800", "UK 🇬🇧"),
            263 => ("G99-TYPEC-HV-MV800", "UK 🇬🇧"),
            264 => ("G99-TYPED-MV800", "UK 🇬🇧"),
            265 => ("G99-TYPEA-HV", "UK 🇬🇧"),
            266 => ("CEA-MV800", "India 🇮🇳"),
            267 => ("EN50549-MV400", "Europe 🇪🇺"),
            268 => ("VDE-AR-N4110", "Germany 🇩🇪"),
            269 => ("VDE-AR-N4110-MV480", "Germany 🇩🇪"),
            270 => ("VDE-AR-N4110-MV800", "Germany 🇩🇪"),
            271 => ("Panama-MV800", "Panama 🇵🇦"),
            272 => ("North Macedonia-MV800", "North Macedonia 🇲🇰"),
            273 => ("NTS", "Spain 🇪🇸"),
            274 => ("NTS-MV480", "Spain 🇪🇸"),
            275 => ("NTS-MV800", "Spain 🇪🇸"),
            276 => ("AS4777-WP", "Australia 🇦🇺"),
            277 => ("CEA", "India 🇮🇳"),
            278 => ("CEA-MV480", "India 🇮🇳"),
            279 => ("SINGAPORE", "Singapore 🇸🇬"),
            280 => ("SINGAPORE-MV480", "Singapore 🇸🇬"),
            281 => ("SINGAPORE-MV800", "Singapore 🇸🇬"),
            282 => ("HONGKONG", "Hong Kong 🇭🇰"),
            283 => ("HONGKONG-MV480", "Hong Kong 🇭🇰"),
            284 => ("C10/11-MV400", "Belgium 🇧🇪"),
            285 => ("KOREA-MV800", "Korea 🇰🇷"),
            286 => ("Cambodia", "Cambodia 🇰🇭"),
            287 => ("Cambodia-MV480", "Cambodia 🇰🇭"),
            288 => ("Cambodia-MV800", "Cambodia 🇰🇭"),
            289 => ("EN50549-SE", "Sweden 🇸🇪"),
            290 => ("GREG030", "Colombia 🇨🇴"),
            291 => ("GREG030-MV440", "Colombia 🇨🇴"),
            292 => ("GREG030-MV480", "Colombia 🇨🇴"),
            293 => ("GREG060-MV800", "Colombia 🇨🇴"),
            294 => ("PERU-MV800", "Peru 🇵🇪"),
            295 => ("PORTUGAL", "Portugal 🇵🇹"),
            296 => ("PORTUGAL-MV480", "Portugal 🇵🇹"),
            297 => ("AS4777-ACT", "Australia 🇦🇺"),
            298 => ("AS4777-NSW-ESS", "Australia 🇦🇺"),
            299 => ("AS4777-NSW-AG", "Australia 🇦🇺"),
            300 => ("AS4777-QLD", "Australia 🇦🇺"),
            301 => ("AS4777-SA", "Australia 🇦🇺"),
            302 => ("AS4777-VIC", "Australia 🇦🇺"),
            303 => ("EN50549-PL", "Poland 🇵🇱"),
            304 => ("Island-Grid", "General"),
            305 => ("TAIPOWER-LV220", "China Taiwan 🇹🇼"),
            306 => ("Mexico-LV220", "Mexico 🇲🇽"),
            307 => ("ABNT NBR 16149-LV127", "Brazil 🇧🇷"),
            308 => ("Philippines-LV220-50Hz", "Philippines 🇵🇭"),
            309 => ("Philippines-LV220-60Hz", "Philippines 🇵🇭"),
            310 => ("Israel-HV800", "Israel 🇮🇱"),
            311 => ("DENMARK-EN50549-DK1-LV230", "Denmark 🇩🇰"),
            312 => ("DENMARK-EN50549-DK2-LV230", "Denmark 🇩🇰"),
            313 => ("SWITZERLAND-NA/EEA:2020-LV230", "Switzerland 🇨🇭"),
            314 => ("Japan-LV202-50Hz", "Japan 🇯🇵"),
            315 => ("Japan-LV202-60Hz", "Japan 🇯🇵"),
            316 => ("AUSTRIA-MV800", "Austria 🇦🇹"),
            317 => ("AUSTRIA-HV800", "Austria 🇦🇹"),
            318 => ("POLAND-EN50549-MV800", "Poland 🇵🇱"),
            319 => ("IRELAND-EN50549-LV230", "Ireland 🇮🇪"),
            320 => ("IRELAND-EN50549-MV480", "Ireland 🇮🇪"),
            321 => ("IRELAND-EN50549-MV800", "Ireland 🇮🇪"),
            322 => ("DENMARK-EN50549-MV800", "Denmark 🇩🇰"),
            323 => ("FRANCE-RTE-MV800", "France 🇫🇷"),
            324 => ("AUSTRALIA-AS4777_A-LV230", "Australia 🇦🇺"),
            325 => ("AUSTRALIA-AS4777_B-LV230", "Australia 🇦🇺"),
            326 => ("AUSTRALIA-AS4777_C-LV230", "Australia 🇦🇺"),
            327 => ("AUSTRALIA-AS4777_NZ-LV230", "Australia 🇦🇺"),
            328 => ("AUSTRALIA-AS4777_A-MV800", "Australia 🇦🇺"),
            329 => ("CHINA-GBT34120-MV800", "China 🇨🇳"),
            _ => ("unknown", "unknown"),
        };
        format!("standard: <b><cyan>{}</>, country: <b><cyan>{}</>", grid_code.0, grid_code.1)
    }

    #[rustfmt::skip]
    pub fn get_state1_description(code: u16) -> String {
        let mut descr = String::from("");
        let state1_masks = vec! [
            (0b0000_0000_0000_0001, "standby"),
            (0b0000_0000_0000_0010, "grid-connected"),
            (0b0000_0000_0000_0100, "grid-connected normally"),
            (0b0000_0000_0000_1000, "grid connection with derating due to power rationing"),
            (0b0000_0000_0001_0000, "grid connection with derating due to internal causes of the solar inverter"),
            (0b0000_0000_0010_0000, "normal stop"),
            (0b0000_0000_0100_0000, "stop due to faults"),
            (0b0000_0000_1000_0000, "stop due to power rationing"),
            (0b0000_0001_0000_0000, "shutdown"),
            (0b0000_0010_0000_0000, "spot check"),
        ];
        for mask in state1_masks {
            if code & mask.0 > 0 {
                descr = descr.add(mask.1).add(" | ");
            }
        }
        if !descr.is_empty() {
            descr.pop();
            descr.pop();
            descr.pop();
        }
        descr
    }

    #[rustfmt::skip]
    pub fn get_state2_description(code: u16) -> String {
        let mut descr = String::from("");
        let state2_masks = vec! [
            (0b0000_0000_0000_0001, ("locked", "unlocked")),
            (0b0000_0000_0000_0010, ("PV disconnected", "PV connected")),
            (0b0000_0000_0000_0100, ("no DSP data collection", "DSP data collection")),
        ];
        for mask in state2_masks {
            if code & mask.0 > 0 {
                descr = descr.add(mask.1.1).add(" | ");
            } else {
                descr = descr.add(mask.1.0).add(" | ");
            }
        }
        if !descr.is_empty() {
            descr.pop();
            descr.pop();
            descr.pop();
        }
        descr
    }

    #[rustfmt::skip]
    pub fn get_state3_description(code: u32) -> String {
        let mut descr = String::from("");
        let state3_masks = vec! [
            (0b0000_0000_0000_0000_0000_0000_0000_0001, ("on-grid", "off-grid")),
            (0b0000_0000_0000_0000_0000_0000_0000_0010, ("off-grid switch disabled", "off-grid switch enabled",)),
        ];
        for mask in state3_masks {
            if code & mask.0 > 0 {
                descr = descr.add(mask.1.1).add(" | ");
            } else {
                descr = descr.add(mask.1.0).add(" | ");
            }
        }
        if !descr.is_empty() {
            descr.pop();
            descr.pop();
            descr.pop();
        }
        descr
    }

    #[rustfmt::skip]
    pub fn get_alarm1_description(code: u16) -> String {
        let mut descr = String::from("");
        let alarm1_masks = vec! [
            (0b0000_0000_0000_0001, Alarm::new("High String Input Voltage", 2001, "Major")),
            (0b0000_0000_0000_0010, Alarm::new("DC Arc Fault", 2002, "Major")),
            (0b0000_0000_0000_0100, Alarm::new("String Reverse Connection", 2011, "Major")),
            (0b0000_0000_0000_1000, Alarm::new("String Current Backfeed", 2012, "Warning")),
            (0b0000_0000_0001_0000, Alarm::new("Abnormal String Power", 2013, "Warning")),
            (0b0000_0000_0010_0000, Alarm::new("AFCI Self-Check Fail", 2021, "Major")),
            (0b0000_0000_0100_0000, Alarm::new("Phase Wire Short-Circuited to PE", 2031, "Major")),
            (0b0000_0000_1000_0000, Alarm::new("Grid Loss", 2032, "Major")),
            (0b0000_0001_0000_0000, Alarm::new("Grid Undervoltage", 2033, "Major")),
            (0b0000_0010_0000_0000, Alarm::new("Grid Overvoltage", 2034, "Major")),
            (0b0000_0100_0000_0000, Alarm::new("Grid Volt. Imbalance", 2035, "Major")),
            (0b0000_1000_0000_0000, Alarm::new("Grid Overfrequency", 2036, "Major")),
            (0b0001_0000_0000_0000, Alarm::new("Grid Underfrequency", 2037, "Major")),
            (0b0010_0000_0000_0000, Alarm::new("Unstable Grid Frequency", 2038, "Major")),
            (0b0100_0000_0000_0000, Alarm::new("Output Overcurrent", 2039, "Major")),
            (0b1000_0000_0000_0000, Alarm::new("Output DC Component Overhigh", 2040, "Major")),
        ];
        for mask in alarm1_masks {
            if code & mask.0 > 0 {
                descr = descr.add(
                    format!("code={} {:?} severity={}", mask.1.code, mask.1.name, mask.1.severity).as_str()
                ).add(" | ");
            }
        }
        if !descr.is_empty() {
            descr.pop();
            descr.pop();
            descr.pop();
            descr
        } else {
            "None".into()
        }
    }

    #[rustfmt::skip]
    pub fn get_alarm2_description(code: u16) -> String {
        let mut descr = String::from("");
        let alarm2_masks = vec! [
            (0b0000_0000_0000_0001, Alarm::new("Abnormal Residual Current", 2051, "Major")),
            (0b0000_0000_0000_0010, Alarm::new("Abnormal Grounding", 2061, "Major")),
            (0b0000_0000_0000_0100, Alarm::new("Low Insulation Resistance", 2062, "Major")),
            (0b0000_0000_0000_1000, Alarm::new("Overtemperature", 2063, "Minor")),
            (0b0000_0000_0001_0000, Alarm::new("Device Fault", 2064, "Major")),
            (0b0000_0000_0010_0000, Alarm::new("Upgrade Failed or Version Mismatch", 2065, "Minor")),
            (0b0000_0000_0100_0000, Alarm::new("License Expired", 2066, "Warning")),
            (0b0000_0000_1000_0000, Alarm::new("Faulty Monitoring Unit", 61440, "Minor")),
            (0b0000_0001_0000_0000, Alarm::new("Faulty Power Collector", 2067, "Major")),
            (0b0000_0010_0000_0000, Alarm::new("Battery abnormal", 2068, "Minor")),
            (0b0000_0100_0000_0000, Alarm::new("Active Islanding", 2070, "Major")),
            (0b0000_1000_0000_0000, Alarm::new("Passive Islanding", 2071, "Major")),
            (0b0001_0000_0000_0000, Alarm::new("Transient AC Overvoltage", 2072, "Major")),
            (0b0010_0000_0000_0000, Alarm::new("Peripheral port short circuit", 2075, "Warning")),
            (0b0100_0000_0000_0000, Alarm::new("Churn output overload", 2077, "Major")),
            (0b1000_0000_0000_0000, Alarm::new("Abnormal PV module configuration", 2080, "Major")),
        ];
        for mask in alarm2_masks {
            if code & mask.0 > 0 {
                descr = descr.add(
                    format!("code={} {:?} severity={}", mask.1.code, mask.1.name, mask.1.severity).as_str()
                ).add(" | ");
            }
        }
        if !descr.is_empty() {
            descr.pop();
            descr.pop();
            descr.pop();
            descr
        } else {
            "None".into()
        }
    }

    #[rustfmt::skip]
    pub fn get_alarm3_description(code: u16) -> String {
        let mut descr = String::from("");
        let alarm3_masks = vec! [
            (0b0000_0000_0000_0001, Alarm::new("Optimizer fault", 2081, "Warning")),
            (0b0000_0000_0000_0010, Alarm::new("Built-in PID operation abnormal", 2085, "Minor")),
            (0b0000_0000_0000_0100, Alarm::new("High input string voltage to ground", 2014, "Major")),
            (0b0000_0000_0000_1000, Alarm::new("External Fan Abnormal", 2086, "Major")),
            (0b0000_0000_0001_0000, Alarm::new("Battery Reverse Connection", 2069, "Major")),
            (0b0000_0000_0010_0000, Alarm::new("On-grid/Off-grid controller abnormal", 2082, "Major")),
            (0b0000_0000_0100_0000, Alarm::new("PV String Loss", 2015, "Warning")),
            (0b0000_0000_1000_0000, Alarm::new("Internal Fan Abnormal", 2087, "Major")),
            (0b0000_0001_0000_0000, Alarm::new("DC Protection Unit Abnormal", 2088, "Major")),
        ];
        for mask in alarm3_masks {
            if code & mask.0 > 0 {
                descr = descr.add(
                    format!("code={} {:?} severity={}", mask.1.code, mask.1.name, mask.1.severity).as_str()
                ).add(" | ");
            }
        }
        if !descr.is_empty() {
            descr.pop();
            descr.pop();
            descr.pop();
            descr
        } else {
            "None".into()
        }
    }

    pub fn set_new_status(
        &mut self,
        thread_name: &String,
        device_status: Option<u16>,
        storage_status: Option<i16>,
        grid_code: Option<u16>,
        state_1: Option<u16>,
        state_2: Option<u16>,
        state_3: Option<u32>,
        alarm_1: Option<u16>,
        alarm_2: Option<u16>,
        alarm_3: Option<u16>,
        fault_code: Option<u16>,
        changes: &mut HashMap<&str, String>
    ) -> bool {
        let mut failure = false;

        if device_status.is_some() && self.device_status != device_status {
            let l = Sun2000State::get_device_status_description(device_status.unwrap());
            changes.insert("status", l.into());

            info!(
                "<i>{}</>: status: <b>{}</>",
                thread_name,
                &l
            );
            self.device_status = device_status;
        }
        if fault_code.is_some() && self.fault_code != fault_code {
            changes.insert("fault_code", fault_code.unwrap().to_string());

            info!(
                "<i>{}</>: fault_code: <b>{}</>",
                thread_name,
                fault_code.unwrap().to_string()
            );
            self.fault_code = fault_code;
        }
        if storage_status.is_some() && self.storage_status != storage_status {
            let l = Sun2000State::get_storage_status_description(storage_status.unwrap());
            changes.insert("storage_status", l.into());
            info!(
                "<i>{}</>: storage status: <b>{}</>",
                thread_name,
                l
            );
            self.storage_status = storage_status;
        }
        if grid_code.is_some() && self.grid_code != grid_code {
            let l = Sun2000State::get_grid_code_description(grid_code.unwrap());
            changes.insert("grid_code", l.clone());
            info!(
                "<i>{}</>: grid: <b>{}</>",
                thread_name,
                l
            );
            self.grid_code = grid_code;
        }
        if state_1.is_some() && self.state_1 != state_1 {
            let l = Sun2000State::get_state1_description(state_1.unwrap());
            changes.insert("state_1", l.clone());
            info!(
                "<i>{}</>: state_1: <b>{}</>",
                thread_name,
                l.clone()
            );
            self.state_1 = state_1;
        }
        if state_2.is_some() && self.state_2 != state_2 {
            let l = Sun2000State::get_state2_description(state_2.unwrap());
            changes.insert("state_2", l.clone());
            info!(
                "<i>{}</>: state_2: <b>{}</>",
                thread_name,
                l.clone()
            );
            self.state_2 = state_2;
        }
        if state_3.is_some() && self.state_3 != state_3 {
            let l = Sun2000State::get_state3_description(state_3.unwrap());
            changes.insert("state_3", l.clone());
            info!(
                "<i>{}</>: state_3: <b>{}</>",
                thread_name,
                l.clone()
            );
            self.state_3 = state_3;
        }
        if alarm_1.is_some() && self.alarm_1 != alarm_1 {
            if alarm_1.unwrap() != 0 || self.alarm_1.is_some() {
                let l =  Sun2000State::get_alarm1_description(alarm_1.unwrap());
                changes.insert("alarm_1", l.clone());
                warn!(
                    "<i>{}</>: alarm_1: <b><red>{}</>",
                    thread_name,
                    l.clone()
                );
            }
            self.alarm_1 = alarm_1;
            failure = alarm_1.unwrap() != 0;
        }
        if alarm_2.is_some() && self.alarm_2 != alarm_2 {
            if alarm_2.unwrap() != 0 || self.alarm_2.is_some() {
                let l = Sun2000State::get_alarm2_description(alarm_2.unwrap());
                changes.insert("alarm_2", l.clone());
                warn!(
                    "<i>{}</>: alarm_2: <b><red>{}</>",
                    thread_name,
                    l.clone()
                );
            }
            self.alarm_2 = alarm_2;
            failure = alarm_2.unwrap() != 0;
        }
        if alarm_3.is_some() && self.alarm_3 != alarm_3 {
            if alarm_3.unwrap() != 0 || self.alarm_3.is_some() {
                let l = Sun2000State::get_alarm3_description(alarm_3.unwrap());
                changes.insert("alarm_3", l.clone());
                warn!(
                    "<i>{}</>: alarm_3: <b><red>{}</>",
                    thread_name,
                    l.clone()
                );
            }
            self.alarm_3 = alarm_3;
            failure = alarm_3.unwrap() != 0;
        }
        failure
    }
}

pub fn get_attribute_name(id: &str) -> &'static str {
    let device_description_attributes = vec![
        (1, "Device model"),
        (2, "Device software version"),
        (3, "Port protocol version"),
        (4, "ESN"),
        (5, "Device ID"),
        (6, "Feature version"),
    ];
    if let Ok(id) = id.parse::<u8>() {
        for elem in device_description_attributes {
            if elem.0 == id {
                return elem.1;
            }
        }
    }
    "Unknown attribute"
}
