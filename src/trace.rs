use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use bincode::config::standard;
use bincode::serde::{decode_from_std_read, encode_into_std_write};

#[derive(Serialize, Deserialize, Clone)]
pub struct RunMeta {
    pub sim_hz: u32,
    pub seed: u64,
    pub version: u32, // bump if you change event schema
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PropsDelta {
    // You can store Entity directly thanks to your helper:
    #[serde(with = "crate::entity_serde::opt")]
    pub selected_entity: Option<hecs::Entity>,
    pub draw_map_grid: Option<bool>,
    pub quit: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TickEvents {
    pub tick: u64,
    pub props: Option<PropsDelta>,
    pub commands: Vec<crate::entity_commands::EntityCommand>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Trailer {
    pub end_tick: u64,
    pub final_world_hash: u64,
}

pub struct Recorder {
    cfg: bincode::config::Configuration,
    w: BufWriter<File>,
}

impl Recorder {
    pub fn new(path: impl AsRef<Path>, meta: RunMeta) -> Result<Self> {
        let f = File::create(path)?;
        let mut w = BufWriter::new(f);
        let cfg = standard();
        encode_into_std_write(&meta, &mut w, cfg)?;
        Ok(Self { cfg, w })
    }
    pub fn push(&mut self, ev: &TickEvents) -> Result<()> {
        encode_into_std_write(ev, &mut self.w, self.cfg)?;
        Ok(())
    }
    pub fn finish(mut self, trailer: &Trailer) -> Result<()> {
        encode_into_std_write(trailer, &mut self.w, self.cfg)?;
        Ok(())
    }
}

pub struct Player {
    cfg: bincode::config::Configuration,
    r: BufReader<File>,
    pub meta: RunMeta,
    peek: Option<TickEvents>,
}

impl Player {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let f = File::open(path)?;
        let mut r = BufReader::new(f);
        let cfg = standard();
        let meta: RunMeta = decode_from_std_read(&mut r, cfg)?; // D inferred from assignment
        Ok(Self {
            cfg,
            r,
            meta,
            peek: None,
        })
    }

    pub fn next_for_tick(&mut self, tick: u64) -> Result<Vec<TickEvents>> {
        let mut out = Vec::new();

        if let Some(ev) = self.peek.take() {
            if ev.tick == tick {
                out.push(ev);
            } else {
                self.peek = Some(ev);
            }
        }

        while self.peek.is_none() {
            match decode_from_std_read::<TickEvents, _, _>(&mut self.r, self.cfg) {
                Ok(ev) if ev.tick == tick => out.push(ev),
                Ok(ev) => {
                    self.peek = Some(ev);
                    break;
                }
                Err(_) => break, // EOF or trailer; handled by read_trailer()
            }
        }

        Ok(out)
    }

    pub fn read_trailer(mut self) -> Result<Trailer> {
        let tr: Trailer = decode_from_std_read(&mut self.r, self.cfg)?; // D inferred from assignment
        Ok(tr)
    }
}
