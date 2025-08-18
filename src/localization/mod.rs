use std::{fs::File, io::Read, sync::{LazyLock, Mutex, MutexGuard}};

use unic_langid::LanguageIdentifier;

use fluent::{concurrent::FluentBundle, FluentResource};

static LOCALIZATION: LazyLock<Mutex<Localization>> = LazyLock::new(|| Mutex::new(Localization::new()));

pub(crate) fn localization() -> MutexGuard<'static, Localization> {
    LOCALIZATION.lock().unwrap()
}

pub(crate) struct Localization {
    bundle: FluentBundle<FluentResource>
}

impl Localization {

    fn new() -> Self {
        let langid_en: LanguageIdentifier = "en-US".parse().expect("Parsing failed");
        let bundle = load_bundle(langid_en.clone());
        Self {
            bundle
        }
    }

    pub(crate) fn localize(&self, message: &str) -> String {
        return self.try_localize(message).unwrap_or(message.to_string());
    }

    pub(crate) fn try_localize(&self, message: &str) -> Option<String> {
        let msg = self.bundle.get_message(message)?;
        let mut errors = vec![];
        let pattern = msg.value()?;
        let value = self.bundle.format_pattern(&pattern, None, &mut errors);
        return Some(value.to_string());
    }

}

fn load_bundle(langid: LanguageIdentifier) -> FluentBundle<FluentResource> {
    let loc_str = langid.to_string();
    let mut loc_file = File::open(&format!("./assets/localization/{loc_str}.ftl"))
        .expect("Locale file not found");
    
    let mut ftl_string = String::new();
    loc_file.read_to_string(&mut ftl_string)
        .expect("Unable to read the file");

    let res = FluentResource::try_new(ftl_string)
        .expect("Failed to parse an FTL string.");

    let mut bundle = FluentBundle::new_concurrent(vec![langid]);

    bundle
        .add_resource(res)
        .expect("Failed to add FTL resources to the bundle.");

    return bundle;
}

#[macro_export]
macro_rules! loc {
    ($($arg:tt)*) => {{
        &crate::localization::localization().localize($($arg)*)
    }};
}
