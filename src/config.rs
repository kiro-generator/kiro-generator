mod agent;
mod agent_file;
mod hook;
mod mcp;
mod merge;
mod native;

pub use agent::{KdlAgent, KdlAgentDoc};
use {
    crate::Fs,
    facet::Facet,
    facet_kdl as kdl,
    miette::IntoDiagnostic,
    std::{
        collections::{HashMap, HashSet},
        fmt::Debug,
        path::Path,
    },
};

pub(crate) type ConfigResult<T> = miette::Result<T>;

#[derive(Facet, Debug, Default, PartialEq, Clone, Eq, Hash)]
#[facet(default)]
pub(super) struct GenericItem {
    #[facet(kdl::argument)]
    pub item: String,
}

#[derive(Facet, Debug, Default, PartialEq, Clone, Eq)]
#[facet(default)]
pub(super) struct GenericSet {
    #[facet(kdl::arguments)]
    pub item: HashSet<String>,
}

#[derive(Facet, Debug, Default, PartialEq, Clone, Eq)]
#[facet(default)]
pub(super) struct GenericVec {
    #[facet(kdl::arguments)]
    pub item: Vec<String>,
}

impl From<GenericVec> for HashMap<String, String> {
    fn from(list: GenericVec) -> HashMap<String, String> {
        list.item
            .chunks_exact(2)
            .map(|chunk| (chunk[0].clone(), chunk[1].clone()))
            .collect()
    }
}

#[cfg(test)]
impl GenericVec {
    fn len(&self) -> usize {
        self.item.len()
    }
}

#[derive(Facet, Copy, Default, Clone, Debug, PartialEq, Eq)]
#[facet(default)]
pub(super) struct IntDoc {
    #[facet(kdl::argument)]
    pub value: u64,
}

#[cfg(test)]
fn print_error(e: &facet_kdl::KdlDeserializeError) {
    // let d = e.into_diagnostics();
    eprintln!("\n=== Miette render ===");
    let mut output = String::new();
    let handler = miette::GraphicalReportHandler::new_themed(miette::GraphicalTheme::unicode());
    handler.render_report(&mut output, e).unwrap();
    eprintln!("{}", output);
}

pub(super) fn split_newline(list: Vec<GenericItem>) -> HashSet<String> {
    list.iter()
        .flat_map(|f| f.item.split('\n'))
        .map(str::trim)
        .filter(|s| !s.is_empty() && s.is_ascii())
        .map(String::from)
        .collect()
}

pub fn kdl_parse_path<T>(fs: &Fs, path: impl AsRef<Path>) -> Option<ConfigResult<T>>
where
    T: for<'a> facet::Facet<'a>,
{
    if fs.exists(&path) {
        match fs.read_to_string_sync(&path).into_diagnostic() {
            Err(e) => Some(Err(e)),
            Ok(content) => match kdl::from_str::<T>(&content) {
                Err(e) => {
                    let kdl_err =
                        &crate::Error::DeserializeError(path.as_ref().display().to_string(), e);
                    crate::output::print_error(kdl_err);
                    Some(Err(crate::format_err!("{kdl_err}")))
                }
                Ok(r) => Some(Ok(r)),
            },
        }
    } else {
        None
    }
}

#[cfg(test)]
pub(crate) fn kdl_parse<T>(content: &str) -> ConfigResult<T>
where
    T: for<'a> facet::Facet<'a>,
{
    match kdl::from_str::<T>(content) {
        Err(e) => {
            print_error(&e);
            Err(crate::format_err!("{e}"))
        }
        Ok(r) => Ok(r),
    }
}

#[derive(facet::Facet, Default)]
pub struct GeneratorConfigDoc {
    #[facet(facet_kdl::children, default)]
    pub agents: Vec<KdlAgentDoc>,
}

#[derive(Default)]
pub struct GeneratorConfig {
    pub agents: HashMap<String, KdlAgent>,
}

impl From<GeneratorConfigDoc> for GeneratorConfig {
    fn from(value: GeneratorConfigDoc) -> Self {
        let mut agent: HashMap<String, KdlAgent> = HashMap::with_capacity(value.agents.len());
        for a in value.agents {
            agent.insert(a.name.clone(), a.into());
        }
        Self { agents: agent }
    }
}

impl Debug for GeneratorConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "agents={}", self.agents.len())
    }
}

impl GeneratorConfig {
    pub fn get(&self, name: impl AsRef<str>) -> Option<&KdlAgent> {
        self.agents.get(name.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use {super::*, crate::config::agent_file::KdlAgentFileDoc};

    #[test_log::test]
    fn test_agent_decoding() -> ConfigResult<()> {
        let kdl_agents = indoc::indoc! {r#"
            agent "test" include-mcp-json=#true {
                inherits "parent"
                description "This is a test agent"
                prompt "Generate a test prompt"
                resource "file://resource.md"
                resource "file://README.md"
                tools "*"
                allowed-tools "@awsdocs"
                hook {
                    agent-spawn "spawn" {
                        command "echo i have spawned"
                        timeout-ms 1000
                        max-output-size 9000
                        cache-ttl-seconds 2
                    }
                    user-prompt-submit "submit" {
                        command "echo user submitted"
                    }
                    pre-tool-use "pre" {
                        command "echo before tool"
                        matcher "git.*"
                    }
                    post-tool-use "post" {
                        command "echo after tool"
                    }
                    stop "stop" {
                        command "echo stopped"
                    }
                }

                mcp "awsdocs" {
                   command "aws-docs"
                   args """
                   --verbose
                   --config=/path
                   """
                   env "RUST_LOG" "debug"
                   env "PATH" "/usr/bin"
                   header "Authorization" "Bearer token"
                   timeout 5000
                }

                alias "execute_bash" "shell"

                native-tool {
                   write {
                       allow "./src/*"
                       allow "./scripts/**"
                       deny  "Cargo.lock"
                       override "/tmp"
                       override "/var/log"
                   }
                   shell deny-by-default=#true {
                      allow "git status .*"
                      deny "git push .*"
                      override "git pull .*"
                   }
                }
            }
        "#};

        let config: GeneratorConfigDoc = kdl_parse(kdl_agents)?;
        assert_eq!(config.agents.len(), 1);
        let config = GeneratorConfig::from(config);
        let agent = config.agents.get("test");
        assert!(agent.is_some());
        let agent = agent.unwrap().clone();
        assert_eq!(agent.name, "test");
        assert!(agent.model.is_none());
        assert!(!agent.is_template());
        let inherits = &agent.inherits;
        assert_eq!(inherits.len(), 1);
        assert_eq!(inherits.iter().next().unwrap(), "parent");
        assert!(agent.description.is_some());
        assert!(agent.prompt.is_some());
        assert!(agent.include_mcp_json.unwrap_or_default());
        let tools = &agent.tools;
        assert_eq!(tools.len(), 1);
        assert_eq!(tools.iter().next().unwrap(), "*");
        let resources = &agent.resources;
        assert_eq!(resources.len(), 2);
        assert!(resources.contains(&"file://resource.md".to_string()));
        assert!(resources.contains(&"file://README.md".to_string()));

        let hooks = &agent.hook;
        let hook = &hooks.agent_spawn.get("spawn");
        assert!(hook.is_some());
        let hook = hook.unwrap();
        assert_eq!(hook.command, "echo i have spawned");

        // assert!(hooks.contains_key(&HookTrigger::PreToolUse));
        // assert!(hooks.contains_key(&HookTrigger::PostToolUse));
        // assert!(hooks.contains_key(&HookTrigger::Stop));
        // assert!(hooks.contains_key(&HookTrigger::UserPromptSubmit));

        let allowed = &agent.allowed_tools;
        assert_eq!(allowed.len(), 1);
        assert_eq!(allowed.iter().next().unwrap(), "@awsdocs");

        let mcp = &agent.mcp;
        assert_eq!(mcp.len(), 1);
        assert!(mcp.contains_key("awsdocs"));
        let aws_docs = mcp.get("awsdocs").unwrap();
        assert_eq!(aws_docs.command, "aws-docs");
        assert_eq!(aws_docs.args, vec!["--verbose\n--config=/path"]);
        assert!(!aws_docs.disabled);
        assert_eq!(aws_docs.headers.len(), 1);
        assert_eq!(aws_docs.env.len(), 2);
        assert_eq!(aws_docs.timeout, 5000);
        assert_eq!(agent.alias.len(), 1);

        Ok(())
    }

    #[test_log::test]
    fn test_agent_empty() -> ConfigResult<()> {
        let kdl_agents = r#"
            agent "test" template=#true {
            }
        "#;

        let config: GeneratorConfigDoc = kdl_parse(kdl_agents)?;
        let config = GeneratorConfig::from(config);
        assert!(!format!("{config:?}").is_empty());
        assert_eq!(config.agents.len(), 1);
        let agent = config.agents.get("test").unwrap();
        assert_eq!(agent.name, "test");
        assert!(agent.model.is_none());
        assert!(agent.is_template());

        Ok(())
    }

    #[test_log::test]
    fn test_agent_file_source() -> ConfigResult<()> {
        let kdl_agent_file_source = r#"
            description "agent from file"
            prompt "Generate a test prompt"
            resource "file://resource.md"
            resource "file://README.md"
            include-mcp-json #true
            tools "*"

            allowed-tools "@awsdocs"
            hook {
                agent-spawn "spawn" {
                    command "echo i have spawned"
                    timeout-ms 1000
                    max-output-size 9000
                    cache-ttl-seconds 2
                }
                user-prompt-submit "submit" {
                    command "echo user submitted"
                }
                pre-tool-use "pre" {
                    command "echo before tool"
                    matcher "git.*"
                }
                post-tool-use "post" {
                    command "echo after tool"
                }
                stop "stop" {
                    command "echo stopped"
                }
            }

            mcp "awsdocs" {
               command "aws-docs"
               args """
               --verbose
               --config=/path
               """
               env "RUST_LOG" "debug"
               env "PATH" "/usr/bin"
               header "Authorization" "Bearer token"
               timeout 5000
            }

            alias "execute_bash" "shell"

            native-tool {
               write {
                   allow "./src/*"
                   allow "./scripts/**"
                   deny  "Cargo.lock"
                   override "/tmp"
                   override "/var/log"
               }
               shell deny-by-default=#true {
                  allow "git status .*"
                  deny "git push .*"
                  override "git pull .*"
               }
            }
            "#;

        let agent: KdlAgentFileDoc = kdl_parse(kdl_agent_file_source)?;
        assert_eq!(
            agent.description.unwrap_or_default().to_string(),
            "agent from file"
        );

        Ok(())
    }

    #[test_log::test]
    fn test_tool_setting_invalid_json() -> ConfigResult<()> {
        let _kdl = r#"
            agent "test" {
                tool-setting "bad" {
                    json "{ invalid json }"
                }
            }
        "#;
        // TODO
        // let config: GeneratorConfig = kdl_parse(kdl)?;
        // let result = config.agents[0].extra_tool_settings();
        // assert!(result.is_err());
        // assert!(
        //     result
        //         .unwrap_err()
        //         .to_string()
        //         .contains("Failed to parse JSON")
        // );
        Ok(())
    }
}
