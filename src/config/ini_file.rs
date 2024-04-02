use std::fs;

use configparser::ini::Ini;

/// Config file lookup paths. First entries are prioritised.
/// Checks whether the file is readable before moving on but does not coplain
const CONFIG_PATHS: &'static [&'static str] = &[
    #[cfg(test)]
    "./striczkof-io.conf",
    "/usr/local/etc/striczkof-io.conf",
    "/etc/striczkof-io.conf",
];

pub(super) struct Entries {
    
}

impl Entries {
    pub(super) fn new() -> Entries {
        let mut entries = Ini::new();
        entries.load("").expect("bruh");

        Entries {

        }

    }

    
}

#[cfg(test)]
mod tests {

}