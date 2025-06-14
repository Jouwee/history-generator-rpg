pub(crate) fn level_to_xp(level: u16) -> u32 {
    if level == 1 {
        return 0;
    }
    if level == 2 {
        return 300;
    }
    if level == 3 {
        return 900;
    }
    if level == 4 {
        return 2700;
    }
    if level == 5 {
        return 6500;
    }
    if level == 6 {
        return 14000;
    }
    if level == 7 {
        return 23000;
    }
    if level == 8 {
        return 34000;
    }
    if level == 9 {
        return 48000;
    }
    if level == 10 {
        return 64000;
    }
    if level == 11 {
        return 85000;
    }
    if level == 12 {
        return 100000;
    }
    if level == 13 {
        return 120000;
    }
    if level == 14 {
        return 140000;
    }
    if level == 15 {
        return 165000;
    }
    if level == 16 {
        return 195000;
    }
    if level == 17 {
        return 225000;
    }
    if level == 18 {
        return 265000;
    }
    if level == 19 {
        return 305000;
    }
    return 355000
}

pub(crate) fn xp_to_level(xp: u32) -> u16 {
    if xp < 300 {
        return 1;
    }
    if xp < 900 {
        return 2;
    }
    if xp < 2700 {
        return 3;
    }
    if xp < 6500 {
        return 4;
    }
    if xp < 14000 {
        return 5;
    }
    if xp < 23000 {
        return 6;
    }
    if xp < 34000 {
        return 7;
    }
    if xp < 48000 {
        return 8;
    }
    if xp < 64000 {
        return 9;
    }
    if xp < 85000 {
        return 10;
    }
    if xp < 100000 {
        return 11;
    }
    if xp < 120000 {
        return 12;
    }
    if xp < 140000 {
        return 13;
    }
    if xp < 165000 {
        return 14;
    }
    if xp < 195000 {
        return 15;
    }
    if xp < 225000 {
        return 16;
    }
    if xp < 265000 {
        return 17;
    }
    if xp < 305000 {
        return 18;
    }
    if xp < 355000 {
        return 19;
    }
    return 20;
}

