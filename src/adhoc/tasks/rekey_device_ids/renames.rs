use super::rekey::Rekey;

const fn rekey(from: &'static str, to: &'static str) -> Rekey {
    Rekey { from, to }
}

pub const DEVICES: &[Rekey] = &[
    rekey("control-switch", "button-styrbar-1"),
    rekey("small-switch", "button-wxkg11lm-1"),
    rekey("front-door", "door-mccgq12lm-1"),
    rekey("garage-door", "door-mccgq12lm-2"),
    rekey("living-room-epd", "display-epd-1"),
    rekey("hallway-epd", "display-epd-2"),
    rekey("fridge-trmnl", "display-trmnl-1"),
    rekey("env-living-room", "env-wsdcgq11lm-1"),
    rekey("env-bathroom", "env-wsdcgq11lm-2"),
    rekey("env-bedroom", "env-vindstyrka-1"),
    rekey("env-living-room-air", "env-vindstyrka-1"),
    rekey("env-outdoor", "env-wsdcgq12lm-1"),
    rekey("closet-light", "light-t1-1"),
    rekey("table-lamp-bedroom", "light-hue-e27-1"),
    rekey("floor-lamp-living-room", "light-tradfri-1"),
    rekey("living-room-tv", "media-tv-1"),
    rekey("living-room-speaker", "media-cast1-1"),
    rekey("living-room-plant", "plant-plt1b-1"),
    rekey("livingroom-motion", "presence-mtr1-1"),
    rekey("closet-presence", "presence-fp1e-1"),
    rekey("living-room-table-lamp", "plug-ts011f-1"),
    rekey("roborock", "vacuum-roborock-1"),
    rekey("valetudo", "vacuum-rockrobo-1"),
];

pub const ENVIRONMENTS: &[Rekey] = &[
    rekey("living-room", "env-wsdcgq11lm-1"),
    rekey("bathroom", "env-wsdcgq11lm-2"),
    rekey("bedroom", "env-vindstyrka-1"),
    rekey("living-room-air", "env-vindstyrka-1"),
    rekey("outdoor", "env-wsdcgq12lm-1"),
    rekey("living-room-mtr-1", "presence-mtr1-1"),
    rekey("living-room-plant", "plant-plt1b-1"),
];

pub const PLANTS: &[Rekey] = &[rekey("living-room-plant", "plant-plt1b-1")];

pub const DOORS: &[Rekey] = &[
    rekey("front-door", "door-mccgq12lm-1"),
    rekey("garage-door", "door-mccgq12lm-2"),
];
