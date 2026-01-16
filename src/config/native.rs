use {
    crate::agent::{
        AwsTool as KiroAwsTool,
        ExecuteShellTool as KiroShellTool,
        ReadTool as KiroReadTool,
        WriteTool as KiroWriteTool,
    },
    facet::Facet,
    std::collections::HashSet,
};

macro_rules! define_tool {
    ($name:ident) => {
        #[derive(Facet, Clone, Debug, Default, PartialEq, Eq)]
        #[facet(default, deny_unknown_fields)]
        pub struct $name {
            #[facet(default, rename = "allow")]
            pub allows: HashSet<String>,
            #[facet(default, rename = "deny")]
            pub denies: HashSet<String>,
            #[facet(default, rename = "forceAllow")]
            pub force_allow: HashSet<String>,
            #[facet(default, rename = "disableAutoReadOnly")]
            pub disable_auto_readonly: Option<bool>,
            #[facet(default, rename = "denyByDefault")]
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
                if !other.force_allow.is_empty() {
                    tracing::trace!(
                        tool = stringify!($name),
                        count = other.force_allow.len(),
                        "merging force_allow"
                    );
                    self.force_allow.extend(other.force_allow);
                }
                self.disable_auto_readonly =
                    self.disable_auto_readonly.or(other.disable_auto_readonly);
                self.deny_by_default = self.deny_by_default.or(other.deny_by_default);
                self
            }
        }
    };
}

define_tool!(ExecuteShellTool);
define_tool!(AwsTool);
define_tool!(WriteTool);
define_tool!(ReadTool);

#[derive(Facet, Default, Clone, Debug, PartialEq, Eq)]
#[facet(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct NativeTools {
    #[facet(default)]
    pub shell: ExecuteShellTool,
    #[facet(default)]
    pub aws: AwsTool,
    #[facet(default)]
    pub read: ReadTool,
    #[facet(default)]
    pub write: WriteTool,
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
        if !write.force_allow.is_empty() {
            tracing::trace!(
                "Override/Forcing write: {:?}",
                write.force_allow.iter().collect::<Vec<_>>()
            );
            for cmd in write.force_allow.iter() {
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
        if !read.force_allow.is_empty() {
            tracing::trace!(
                "Override/Forcing read: {:?}",
                read.force_allow.iter().collect::<Vec<_>>()
            );
            for cmd in read.force_allow.iter() {
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

        if !shell.force_allow.is_empty() {
            tracing::trace!(
                "Override/Forcing commands: {:?}",
                shell.force_allow.iter().collect::<Vec<_>>()
            );
            for cmd in shell.force_allow.iter() {
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
            config::{ConfigResult, toml_parse},
        },
        std::fmt::Display,
    };
    fn into_set<T: Display>(v: Vec<T>) -> HashSet<String> {
        HashSet::from_iter(v.into_iter().map(|t| t.to_string()))
    }
    #[test_log::test]
    fn parse_shell_tool() -> ConfigResult<()> {
        let raw = r#"
[shell]
denyByDefault=true
disableAutoReadOnly=false
allow = ["ls .*",  "git status"]
deny = ["rm -rf /"]
forceAllow = ["git push"]
        "#;

        let doc: NativeTools = toml_parse(raw)?;
        let shell = doc.shell;
        assert_eq!(shell.allows.len(), 2);
        assert_eq!(shell.denies.len(), 1);
        assert!(shell.deny_by_default.unwrap_or_default());
        assert!(!shell.disable_auto_readonly.unwrap_or_default());
        assert_eq!(shell.force_allow.len(), 1);
        Ok(())
    }

    #[test_log::test]
    fn parse_aws_tool() -> ConfigResult<()> {
        let raw = r#"
            [aws]
            disableAutoReadOnly=true
            allow =  ["ec2" , "s3"]
            deny = ["iam"]
        "#;

        let doc: NativeTools = toml_parse(raw)?;
        let aws = doc.aws;
        assert!(aws.disable_auto_readonly.is_some());
        assert!(aws.disable_auto_readonly.unwrap_or_default());
        assert_eq!(aws.allows.len(), 2);
        assert_eq!(aws.denies.len(), 1);
        Ok(())
    }

    #[test_log::test]
    fn parse_read_write_tools() -> ConfigResult<()> {
        let raw = r#"
            [read]
            allow= ["*.rs", "*.toml"]
            deny= ["/etc/*"]
            forceAllow = ["/etc/hosts"]

            [write]
            allow= ["*.txt"]
            deny= ["/tmp/*"]
            forceAllow = ["/tmp/allowed"]

        "#;

        let doc: NativeTools = toml_parse(raw)?;
        assert_eq!(doc.read.allows.len(), 2);
        assert_eq!(doc.read.denies.len(), 1);
        assert_eq!(doc.read.force_allow.len(), 1);
        assert_eq!(doc.write.allows.len(), 1);
        assert_eq!(doc.write.denies.len(), 1);
        assert_eq!(doc.write.force_allow.len(), 1);
        Ok(())
    }

    #[test_log::test]
    pub fn test_native_merge_empty() -> ConfigResult<()> {
        let child = NativeTools::default();
        let parent = NativeTools::default();
        let merged = child.merge(parent);

        assert_eq!(merged, NativeTools::default());
        toml_parse::<NativeTools>("")?;
        Ok(())
    }

    #[test_log::test]
    pub fn test_native_merge_empty_child() -> Result<()> {
        let child = NativeTools::default();
        let parent = NativeTools {
            aws: AwsTool {
                disable_auto_readonly: None,
                deny_by_default: None,
                force_allow: Default::default(),
                allows: into_set(vec!["ec2"]),
                denies: into_set(vec!["iam"]),
            },
            shell: ExecuteShellTool {
                allows: into_set(vec!["ls .*"]),
                denies: into_set(vec!["git push"]),
                force_allow: into_set(vec!["rm -rf /"]),
                deny_by_default: Some(true),
                disable_auto_readonly: Some(false),
            },
            read: ReadTool {
                allows: into_set(vec!["ls .*"]),
                denies: into_set(vec!["git push"]),
                force_allow: into_set(vec!["rm -rf /"]),
                ..Default::default()
            },
            write: WriteTool {
                allows: into_set(vec!["ls .*"]),
                denies: into_set(vec!["git push"]),
                force_allow: into_set(vec!["rm -rf /"]),
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
                force_allow: into_set(vec!["rm"]),
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
                force_allow: into_set(vec!["rm"]),
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
            force_allow: into_set(vec!["rm"]),
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
