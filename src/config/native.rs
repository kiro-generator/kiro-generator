use {
    super::GenericSet,
    crate::agent::{
        AwsTool as KiroAwsTool,
        ExecuteShellTool as KiroShellTool,
        ReadTool as KiroReadTool,
        WriteTool as KiroWriteTool,
    },
    facet::Facet,
    facet_kdl as kdl,
    std::collections::HashSet,
};

macro_rules! define_tool {
    ($name:ident) => {
        #[derive(Clone, Debug, Default, PartialEq, Eq)]
        pub struct $name {
            pub allows: HashSet<String>,
            pub denies: HashSet<String>,
            pub overrides: HashSet<String>,
            pub disable_auto_readonly: Option<bool>,
            pub deny_by_default: Option<bool>,
        }

        impl $name {
            pub fn merge(mut self, other: Self) -> Self {
                if !other.allows.is_empty() {
                    tracing::trace!(
                        tool = stringify!($name),
                        count = other.allows.len(),
                        "merging allows"
                    );
                    self.allows.extend(other.allows);
                }
                if !other.denies.is_empty() {
                    tracing::trace!(
                        tool = stringify!($name),
                        count = other.denies.len(),
                        "merging denies"
                    );
                    self.denies.extend(other.denies);
                }
                if !other.overrides.is_empty() {
                    tracing::trace!(
                        tool = stringify!($name),
                        count = other.overrides.len(),
                        "merging overrides"
                    );
                    self.overrides.extend(other.overrides);
                }
                self.disable_auto_readonly =
                    self.disable_auto_readonly.or(other.disable_auto_readonly);
                self.deny_by_default = self.deny_by_default.or(other.deny_by_default);
                self
            }
        }
    };
}

macro_rules! define_kdl_doc {
    ($name:ident) => {
        #[derive(Facet, Clone, Debug, Default, PartialEq, Eq)]
        #[facet(default, rename_all = "kebab-case")]
        pub struct $name {
            #[facet(default, kdl::child)]
            pub(super) allows: GenericSet,
            #[facet(default, kdl::child)]
            pub(super) denies: GenericSet,
            #[facet(default, kdl::child)]
            pub(super) overrides: GenericSet,
            #[facet(default, kdl::property)]
            pub deny_by_default: Option<bool>,
            #[facet(default, kdl::property)]
            pub disable_auto_readonly: Option<bool>,
        }
    };
}

macro_rules! define_tool_into {
    ($name:ident, $to:ident) => {
        impl From<$name> for $to {
            fn from(value: $name) -> $to {
                $to {
                    allows: value.allows.item,
                    denies: value.denies.item,
                    overrides: value.overrides.item,
                    deny_by_default: value.deny_by_default,
                    disable_auto_readonly: value.disable_auto_readonly,
                }
            }
        }
    };
}

define_kdl_doc!(AwsToolDoc);
define_kdl_doc!(ExecuteShellToolDoc);
define_kdl_doc!(WriteToolDoc);
define_kdl_doc!(ReadToolDoc);
define_tool!(ExecuteShellTool);
define_tool!(AwsTool);
define_tool!(WriteTool);
define_tool!(ReadTool);
define_tool_into!(ExecuteShellToolDoc, ExecuteShellTool);
define_tool_into!(AwsToolDoc, AwsTool);
define_tool_into!(WriteToolDoc, WriteTool);
define_tool_into!(ReadToolDoc, ReadTool);

#[derive(Facet, Default, Clone, Debug, PartialEq, Eq)]
#[facet(default)]
pub struct NativeToolsDoc {
    #[facet(default, kdl::child)]
    pub shell: ExecuteShellToolDoc,
    #[facet(default, kdl::child)]
    pub aws: AwsToolDoc,
    #[facet(default, kdl::child)]
    pub read: ReadToolDoc,
    #[facet(default, kdl::child)]
    pub write: WriteToolDoc,
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct NativeTools {
    pub shell: ExecuteShellTool,
    pub aws: AwsTool,
    pub read: ReadTool,
    pub write: WriteTool,
}

impl From<NativeToolsDoc> for NativeTools {
    fn from(value: NativeToolsDoc) -> Self {
        Self {
            shell: value.shell.into(),
            aws: value.aws.into(),
            read: value.read.into(),
            write: value.write.into(),
        }
    }
}

impl NativeTools {
    pub fn merge(mut self, other: Self) -> Self {
        self.shell = self.shell.merge(other.shell);
        self.aws = self.aws.merge(other.aws);
        self.read = self.read.merge(other.read);
        self.write = self.write.merge(other.write);
        self
    }
}

impl From<&NativeTools> for KiroAwsTool {
    fn from(value: &NativeTools) -> Self {
        let aws = &value.aws;
        KiroAwsTool {
            allowed_services: aws.allows.clone(),
            denied_services: aws.denies.clone(),
            auto_allow_readonly: !aws.disable_auto_readonly.unwrap_or(false),
        }
    }
}

impl From<&NativeTools> for KiroWriteTool {
    fn from(value: &NativeTools) -> Self {
        let write = &value.write;
        let mut allows: HashSet<String> = write.allows.clone();
        let mut denies: HashSet<String> = write.denies.clone();
        if !write.overrides.is_empty() {
            tracing::trace!(
                "Override/Forcing write: {:?}",
                write.overrides.iter().collect::<Vec<_>>()
            );
            for cmd in write.overrides.iter() {
                allows.insert(cmd.clone());
                if denies.remove(cmd) {
                    tracing::trace!("Removed from denies: {cmd}");
                }
            }
        }

        Self {
            allowed_paths: allows,
            denied_paths: denies,
        }
    }
}

impl From<&NativeTools> for KiroReadTool {
    fn from(value: &NativeTools) -> Self {
        let read = &value.read;
        let mut allows: HashSet<String> = read.allows.clone();
        let mut denies: HashSet<String> = read.denies.clone();
        if !read.overrides.is_empty() {
            tracing::trace!(
                "Override/Forcing write: {:?}",
                read.overrides.iter().collect::<Vec<_>>()
            );
            for cmd in read.overrides.iter() {
                allows.insert(cmd.clone());
                if denies.remove(cmd) {
                    tracing::trace!("Removed from denies: {cmd}");
                }
            }
        }

        Self {
            allowed_paths: allows,
            denied_paths: denies,
        }
    }
}

impl From<&NativeTools> for KiroShellTool {
    fn from(value: &NativeTools) -> Self {
        let shell = &value.shell;
        let mut allows: HashSet<String> = shell.allows.clone();
        let mut denies: HashSet<String> = shell.denies.clone();

        if !shell.overrides.is_empty() {
            tracing::trace!(
                "Override/Forcing commands: {:?}",
                shell.overrides.iter().collect::<Vec<_>>()
            );
            for cmd in shell.overrides.iter() {
                allows.insert(cmd.clone());
                if denies.remove(cmd) {
                    tracing::trace!("Removed command from denies: {cmd}");
                }
            }
        }
        Self {
            allowed_commands: allows,
            denied_commands: denies,
            deny_by_default: shell.deny_by_default.unwrap_or(false),
            auto_allow_readonly: shell.disable_auto_readonly.unwrap_or(true),
        }
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::{
            Result,
            config::{ConfigResult, kdl_parse},
        },
        std::fmt::Display,
    };
    fn into_set<T: Display>(v: Vec<T>) -> HashSet<String> {
        HashSet::from_iter(v.into_iter().map(|t| t.to_string()))
    }
    #[test_log::test]
    fn parse_shell_tool() -> ConfigResult<()> {
        let kdl = r#"
            shell deny-by-default=#true disable-auto-readonly=#false {
                allows "ls .*" "git status"
                denies "rm -rf /"
                overrides "git push"
            }
        "#;

        let doc: NativeToolsDoc = kdl_parse(kdl)?;
        let doc = NativeTools::from(doc);
        let shell = doc.shell;
        assert_eq!(shell.allows.len(), 2);
        assert_eq!(shell.denies.len(), 1);
        assert!(shell.deny_by_default.unwrap_or_default());
        assert!(!shell.disable_auto_readonly.unwrap_or_default());
        assert_eq!(shell.overrides.len(), 1);
        Ok(())
    }

    #[test_log::test]
    fn parse_aws_tool() -> ConfigResult<()> {
        let kdl = r#"
            aws disable-auto-readonly=#true {
                allows "ec2" "s3"
                denies "iam"
            }
        "#;

        let doc: NativeToolsDoc = kdl_parse(kdl)?;
        let aws = NativeTools::from(doc).aws;
        assert!(aws.disable_auto_readonly.is_some());
        assert!(aws.disable_auto_readonly.unwrap_or_default());
        assert_eq!(aws.allows.len(), 2);
        assert_eq!(aws.denies.len(), 1);
        Ok(())
    }

    #[test_log::test]
    fn parse_read_write_tools() -> ConfigResult<()> {
        let kdl = r#"
            read {
                allows "*.rs" "*.toml"
                denies "/etc/*"
                overrides "/etc/hosts"
            }
            write {
                allows "*.txt"
                denies "/tmp/*"
                overrides "/tmp/allowed"
            }
        "#;

        let doc: NativeToolsDoc = kdl_parse(kdl)?;
        let doc = NativeTools::from(doc);
        assert_eq!(doc.read.allows.len(), 2);
        assert_eq!(doc.write.allows.len(), 1);
        Ok(())
    }

    #[test_log::test]
    pub fn test_native_merge_empty() -> Result<()> {
        let child = NativeTools::default();
        let parent = NativeTools::default();
        let merged = child.merge(parent);

        assert_eq!(merged, NativeTools::default());
        Ok(())
    }

    #[test_log::test]
    pub fn test_native_merge_empty_child() -> Result<()> {
        let child = NativeTools::default();
        let parent = NativeTools {
            aws: AwsTool {
                disable_auto_readonly: None,
                deny_by_default: None,
                overrides: Default::default(),
                allows: into_set(vec!["ec2"]),
                denies: into_set(vec!["iam"]),
            },
            shell: ExecuteShellTool {
                allows: into_set(vec!["ls .*"]),
                denies: into_set(vec!["git push"]),
                overrides: into_set(vec!["rm -rf /"]),
                deny_by_default: Some(true),
                disable_auto_readonly: Some(false),
            },
            read: ReadTool {
                allows: into_set(vec!["ls .*"]),
                denies: into_set(vec!["git push"]),
                overrides: into_set(vec!["rm -rf /"]),
                ..Default::default()
            },
            write: WriteTool {
                allows: into_set(vec!["ls .*"]),
                denies: into_set(vec!["git push"]),
                overrides: into_set(vec!["rm -rf /"]),
                ..Default::default()
            },
        };

        let merged = child.merge(parent.clone());
        assert_eq!(merged.aws, parent.aws);
        assert_eq!(merged.shell, parent.shell);
        assert_eq!(merged.read, parent.read);
        assert_eq!(merged.write, parent.write);
        Ok(())
    }

    #[test_log::test]
    pub fn test_native_merge_child_parent() -> Result<()> {
        let child = NativeTools {
            aws: AwsTool {
                disable_auto_readonly: Some(true),
                allows: into_set(vec!["ec2"]),
                ..Default::default()
            },
            ..Default::default()
        };

        let parent = NativeTools {
            aws: AwsTool {
                allows: into_set(vec!["ec2"]),
                denies: into_set(vec!["iam"]),
                ..Default::default()
            },
            ..Default::default()
        };

        let merged = child.merge(parent);
        let aws = merged.aws;
        assert!(aws.disable_auto_readonly.unwrap_or_default());
        // Should have deduplicated ec2
        assert_eq!(aws.allows.len(), 1);
        assert_eq!(aws.denies, into_set(vec!["iam".to_string()]));
        Ok(())
    }

    #[test_log::test]
    pub fn test_native_merge_shell() -> Result<()> {
        let child = ExecuteShellTool::default();
        let parent = ExecuteShellTool {
            deny_by_default: Some(false),
            disable_auto_readonly: Some(false),
            ..Default::default()
        };

        let merged = child.clone().merge(parent);
        assert!(!merged.deny_by_default.unwrap_or_default());
        assert!(!merged.disable_auto_readonly.unwrap_or_default());

        let parent = ExecuteShellTool {
            deny_by_default: Some(true),
            disable_auto_readonly: Some(true),
            ..Default::default()
        };
        let merged = child.clone().merge(parent);
        assert!(merged.deny_by_default.unwrap_or_default());
        assert!(merged.disable_auto_readonly.unwrap_or_default());

        let child = ExecuteShellTool {
            deny_by_default: Some(false),
            disable_auto_readonly: Some(false),
            ..Default::default()
        };
        let parent = ExecuteShellTool {
            deny_by_default: Some(true),
            disable_auto_readonly: Some(true),
            ..Default::default()
        };
        let merged = child.merge(parent);
        assert!(!merged.deny_by_default.unwrap_or_default());
        assert!(!merged.disable_auto_readonly.unwrap_or_default());
        Ok(())
    }

    #[test_log::test]
    pub fn test_native_aws_kiro() -> Result<()> {
        let a = NativeTools::default();
        let kiro = KiroAwsTool::from(&a);
        assert!(kiro.auto_allow_readonly);
        assert!(kiro.allowed_services.is_empty());
        assert!(kiro.denied_services.is_empty());

        let a = NativeTools {
            aws: AwsTool {
                disable_auto_readonly: Some(true),
                allows: into_set(vec!["blah"]),
                denies: into_set(vec!["blahblah"]),
                ..Default::default()
            },
            ..Default::default()
        };

        let kiro = KiroAwsTool::from(&a);
        assert!(!kiro.auto_allow_readonly);
        assert!(kiro.allowed_services.contains("blah"));
        assert!(kiro.denied_services.contains("blahblah"));
        assert_eq!(kiro.allowed_services.len() + kiro.denied_services.len(), 2);
        Ok(())
    }

    #[test_log::test]
    pub fn test_native_shell_kiro() -> Result<()> {
        let a = NativeTools::default();
        let kiro = KiroShellTool::from(&a);
        assert!(kiro.auto_allow_readonly);
        assert!(kiro.allowed_commands.is_empty());
        assert!(kiro.denied_commands.is_empty());

        let a = NativeTools {
            shell: ExecuteShellTool {
                allows: into_set(vec!["ls"]),
                denies: into_set(vec!["rm"]),
                deny_by_default: None,
                disable_auto_readonly: None,
                overrides: into_set(vec!["rm"]),
            },
            ..Default::default()
        };
        let kiro = KiroShellTool::from(&a);
        assert!(kiro.auto_allow_readonly);
        assert_eq!(kiro.allowed_commands.len(), 2);
        assert_eq!(
            kiro.allowed_commands,
            into_set(vec!["ls".to_string(), "rm".to_string()])
        );
        assert!(kiro.denied_commands.is_empty());
        Ok(())
    }

    #[test_log::test]
    pub fn test_native_read_kiro() -> Result<()> {
        let a = NativeTools::default();
        let kiro = KiroReadTool::from(&a);
        assert!(kiro.allowed_paths.is_empty());
        assert!(kiro.denied_paths.is_empty());

        let a = NativeTools {
            read: ReadTool {
                allows: into_set(vec!["ls"]),
                denies: into_set(vec!["rm"]),
                overrides: into_set(vec!["rm"]),
                ..Default::default()
            },
            ..Default::default()
        };
        let kiro = KiroReadTool::from(&a);
        assert_eq!(kiro.allowed_paths.len(), 2);
        assert_eq!(
            kiro.allowed_paths,
            into_set(vec!["ls".to_string(), "rm".to_string()])
        );
        assert!(kiro.denied_paths.is_empty());
        Ok(())
    }

    #[test_log::test]
    pub fn test_native_write_kiro() -> Result<()> {
        let a = NativeTools::default();
        let kiro = KiroWriteTool::from(&a);
        assert!(kiro.allowed_paths.is_empty());
        assert!(kiro.denied_paths.is_empty());
        let write = WriteTool {
            allows: into_set(vec!["ls"]),
            denies: into_set(vec!["rm"]),
            overrides: into_set(vec!["rm"]),
            ..Default::default()
        };
        let a = NativeTools {
            write,
            ..Default::default()
        };

        let kiro = KiroWriteTool::from(&a);
        assert_eq!(kiro.allowed_paths.len(), 2);
        assert_eq!(
            kiro.allowed_paths,
            into_set(vec!["ls".to_string(), "rm".to_string()])
        );
        assert!(kiro.denied_paths.is_empty());
        Ok(())
    }
}
