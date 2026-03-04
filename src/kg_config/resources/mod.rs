mod file;
mod knowledge;
mod skill;

macro_rules! define_location_resource {
    ($name:ident, $scheme:literal) => {
        #[derive(facet::Facet, Clone, Debug, PartialEq, Eq, Default)]
        #[facet(default, skip_all_unless_truthy, deny_unknown_fields)]
        pub struct $name {
            #[facet(default)]
            pub disabled: Option<bool>,
            #[facet(default)]
            pub optional: Option<bool>,
            #[facet(default)]
            pub locations: std::collections::BTreeSet<String>,
        }

        impl $name {
            pub fn merge(mut self, other: Self) -> Self {
                self.locations.extend(other.locations);

                if self.disabled.is_none() {
                    self.disabled = other.disabled;
                }

                if self.optional.is_none() {
                    self.optional = other.optional;
                }

                self
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let out = self
                    .locations
                    .iter()
                    .map(|location| format!("{}://{}", $scheme, location))
                    .collect::<Vec<String>>()
                    .join(",");

                write!(f, "{out}")
            }
        }
    };
}

pub(crate) use define_location_resource;
pub use {file::KgFileResource, knowledge::KgKnowledge, skill::KgSkillResource};
