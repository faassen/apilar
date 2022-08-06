use crate::assembler::Assembler;
use crate::config::RunConfig;
use crate::topology::Topology;
use crate::world::World;
use crate::world::WorldState;
use crate::RunConfigArgs;
use anyhow::anyhow;
use anyhow::Result;
use std::fs::File;
use std::io::BufReader;
use std::time::Duration;

#[derive(Debug)]
pub struct Autosave {
    pub enabled: bool,
    pub frequency: Duration,
}

pub fn load_command(cli: &RunConfigArgs) -> Result<()> {
    let file = BufReader::new(File::open(cli.filename.clone())?);

    let mut archive = zip::ZipArchive::new(file).unwrap();

    let cbor_file = match archive.by_name("data.cbor") {
        Ok(file) => file,
        Err(_) => {
            return Err(anyhow!("data.cbor not found in archive"));
        }
    };

    let run_config: RunConfig = RunConfig::from(cli);
    let world_state: WorldState = serde_cbor::from_reader(cbor_file)?;
    let world = World::from_world_state(world_state);

    let assembler = Assembler::new();

    world.run(run_config, assembler)
}

pub fn run_command(cli: &RunConfigArgs) -> Result<()> {
    let file = BufReader::new(File::open(cli.filename.clone())?);
    let topology: Topology = serde_json::from_reader(file)?;
    let world = World::try_from(&topology)?;
    let run_config = RunConfig::from(cli);
    let assembler = Assembler::new();

    world.run(run_config, assembler)
}
