use {super::*, tracing::enabled};

fn add_local(
    fs: &Fs,
    agent: String,
    raw: String,
    agent_sources: &mut Vec<AgentSource>,
    local_agents: &mut HashSet<String>,
) {
    agent_sources.push(AgentSource::LocalInline(raw));
    let (local, empty) = AgentSource::new_local_agent(&agent, fs);
    if !empty {
        agent_sources.push(local);
        local_agents.insert(agent.clone());
    }
}

/// First pass: Discover all agents from configuration files
///
/// merge agent config from lowest precedence to higher precedence:
/// ```text
/// * `~/.kiro/generators/<agent-name>.toml`
/// * `~/.kiro/generators/kg.toml`
/// * `.kiro/generators/<agent-name>.toml`
/// * `.kiro/generators/kg.toml`
/// ```
#[tracing::instrument(name = "discover", level = "info")]
pub(super) fn agents(
    fs: &Fs,
    location: &ConfigLocation,
) -> Result<(HashMap<String, KgAgent>, HashSet<String>)> {
    location.is_valid(fs)?;

    let global_path = location.global();
    let local_path = location.local();
    let global_exists = fs.exists(&global_path);

    let builder = Config::builder().add_source(
        config::File::from(global_path.clone())
            .required(false)
            .format(config::FileFormat::Toml),
    );
    let global_agents: KgConfig = builder
        .build()
        .wrap_err_with(|| format!("could not process global config: {}", global_path.display()))?
        .try_deserialize()
        .wrap_err_with(|| {
            format!(
                "could not deserialize global path {}",
                global_path.display()
            )
        })?;

    let local_config: KgConfig = Config::builder()
        .add_source(
            config::File::from(local_path.clone())
                .required(false)
                .format(config::FileFormat::Toml),
        )
        .build()
        .wrap_err_with(|| format!("could not process local path {}", local_path.display()))?
        .try_deserialize()
        .wrap_err_with(|| format!("could not deserialize local path {}", local_path.display()))?;

    let mut local_agents = HashSet::from_iter(local_config.agents.keys().cloned());
    tracing::debug!("found {} local agents", local_agents.len());
    let mut all_agents_names: HashSet<String> =
        HashSet::with_capacity(global_agents.agents.keys().len() + local_agents.len());
    all_agents_names.extend(local_agents.clone());
    all_agents_names.extend(global_agents.agents.keys().cloned());

    let mut resolved_agents: HashMap<String, KgAgent> =
        HashMap::with_capacity(all_agents_names.len());

    let global_dir = if global_exists {
        global_path
            .parent()
            .ok_or_else(|| {
                eyre!(
                    "global path does not have parent directory {}",
                    global_path.display()
                )
            })?
            .to_path_buf()
    } else {
        PathBuf::default()
    };
    let mut sources: HashMap<String, Vec<AgentSource>> = HashMap::new();
    for name in all_agents_names {
        sources.insert(name.clone(), Vec::with_capacity(4));
    }
    for (name, agent_sources) in sources.iter_mut() {
        let span = tracing::debug_span!("sources", agent = ?name);
        let _enter = span.enter();
        match location {
            ConfigLocation::Local => {
                add_local(
                    fs,
                    name.to_string(),
                    local_config.get(name)?,
                    agent_sources,
                    &mut local_agents,
                );
            }
            ConfigLocation::Global(_) => {
                // ~/.kiro/generators/<agent-name>.toml
                agent_sources.push(AgentSource::GlobalFile(
                    global_dir.join(format!("{name}.toml")),
                ));
                // ~/.kiro/generators/kg.toml
                agent_sources.push(AgentSource::GlobalInline(global_agents.get(name)?));
            }
            ConfigLocation::Both(_) => {
                // ~/.kiro/generators/<agent-name>.toml
                agent_sources.push(AgentSource::GlobalFile(
                    global_dir.join(format!("{name}.toml")),
                ));
                // ~/.kiro/generators/kg.toml
                agent_sources.push(AgentSource::GlobalInline(global_agents.get(name)?));
                add_local(
                    fs,
                    name.to_string(),
                    local_config.get(name)?,
                    agent_sources,
                    &mut local_agents,
                );
            }
        };
    }
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Agent", "Source1", "Source2", "Source3", "Source4"]);
    for (name, agent_sources) in sources.iter() {
        let mut row: Vec<Cell> = vec![Cell::new(name.to_string())];
        row.extend(agent_sources.iter().map(|s| s.to_cell(fs)));

        // Expand the columns to fill the row as needed, there are 4 sources but not all
        // will be present
        let len: i32 = i32::try_from(row.len())?;
        let expand_slot: i32 = len - 5;
        if expand_slot < 0 {
            let i = len + expand_slot + 1; // name is in row[0] and always exists
            if i >= 0 && i < len {
                let colspan = expand_slot.unsigned_abs() + 1;
                let cell = row[i as usize].clone().set_colspan(colspan as u16);
                row[i as usize] = cell;
            }
        }
        let mut builder = Config::builder();
        for s in agent_sources {
            builder = builder.add_source(s.to_source(fs));
        }
        table.add_row(row);
        let mut agent: KgAgent = builder
            .build()?
            .try_deserialize()
            .wrap_err_with(|| format!("failed to deserialize {name}"))?;
        if tracing::enabled!(tracing::Level::TRACE) {
            tracing::trace!(
                "Deserialized agent: {:?}",
                serde_json::to_string(&agent).unwrap()
            );
        }
        agent.name = name.clone();
        resolved_agents.insert(name.clone(), agent);
    }
    if enabled!(tracing::Level::DEBUG) {
        println!("{table}")
    }
    Ok((resolved_agents, local_agents))
}
