use anyhow::anyhow;
use photon_rs::PhotonImage;
use std::str::FromStr;

#[derive(Debug, Copy, Clone)]
pub(super) enum Filter {
    Cali,
    Dramatic,
    Firenze,
    Golden,
    Lix,
    Lofi,
    Neue,
    Obsidian,
    PastelPink,
    Ryo,
    Oceanic,
    Islands,
    Marine,
    SeaGreen,
    FlagBlue,
    Liquid,
    Diamante,
    Radio,
    Twenties,
    RoseTint,
    Mauve,
    BlueChrome,
    Vintage,
    Perfume,
    Serenity,
}

impl FromStr for Filter {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Filter::*;
        match s.to_lowercase().as_ref() {
            "cali" => Ok(Cali),
            "dramatic" => Ok(Dramatic),
            "firenze" => Ok(Firenze),
            "golden" => Ok(Golden),
            "lix" => Ok(Lix),
            "lofi" => Ok(Lofi),
            "neue" => Ok(Neue),
            "obsidian" => Ok(Obsidian),
            "pastelpink" => Ok(PastelPink),
            "ryo" => Ok(Ryo),
            "oceanic" => Ok(Oceanic),
            "islands" => Ok(Islands),
            "marine" => Ok(Marine),
            "seagreen" => Ok(SeaGreen),
            "flagblue" => Ok(FlagBlue),
            "liquid" => Ok(Liquid),
            "diamante" => Ok(Diamante),
            "radio" => Ok(Radio),
            "twenties" => Ok(Twenties),
            "rosetint" => Ok(RoseTint),
            "mauve" => Ok(Mauve),
            "bluechrome" => Ok(BlueChrome),
            "vintage" => Ok(Vintage),
            "perfume" => Ok(Perfume),
            "serenity" => Ok(Serenity),
            _ => Err(anyhow!("Unknown filter {}", s)),
        }
    }
}

impl Filter {
    pub(super) fn apply(self, image: &mut PhotonImage) {
        use photon_rs::filters::*;
        use Filter::*;
        match self {
            Cali => cali(image),
            Dramatic => dramatic(image),
            Firenze => firenze(image),
            Golden => golden(image),
            Lix => lix(image),
            Lofi => lofi(image),
            Neue => neue(image),
            Obsidian => obsidian(image),
            PastelPink => pastel_pink(image),
            Ryo => ryo(image),
            Oceanic => filter(image, "oceanic"),
            Islands => filter(image, "islands"),
            Marine => filter(image, "marine"),
            SeaGreen => filter(image, "seagreen"),
            FlagBlue => filter(image, "flagblue"),
            Liquid => filter(image, "liquid"),
            Diamante => filter(image, "diamante"),
            Radio => filter(image, "radio"),
            Twenties => filter(image, "twenties"),
            RoseTint => filter(image, "rosetint"),
            Mauve => filter(image, "mauve"),
            BlueChrome => filter(image, "bluechrome"),
            Vintage => filter(image, "vintage"),
            Perfume => filter(image, "perfume"),
            Serenity => filter(image, "serenity"),
        }
    }
}
