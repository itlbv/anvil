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
    pub version: u32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PropsDelta {
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
    pub fn new<P: AsRef<Path>>(path: P, meta: RunMeta) -> Result<Self> {
        let cfg = standard();
        let mut w = BufWriter::new(File::create(path.as_ref())?);
        encode_into_std_write(&meta, &mut w, cfg)?;
        Ok(Self { cfg, w })
    }
    pub fn push(&mut self, ev: &TickEvents) -> Result<()> {
        encode_into_std_write(ev, &mut self.w, self.cfg)?;
        Ok(())
    }
    pub fn finish(mut self, trailer: &Trailer) -> Result<()> {
        use std::io::Write;
        encode_into_std_write(trailer, &mut self.w, self.cfg)?;
        self.w.flush()?;
        Ok(())
    }
}

pub struct Player {
    cfg: bincode::config::Configuration,
    r: BufReader<File>,
    pub meta: RunMeta,
    pub eof_reached: bool,
    pub last_tick_seen: u64,
    peek: Option<TickEvents>,
}

impl Player {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let cfg = standard();
        let mut r = BufReader::new(File::open(path.as_ref())?);
        let meta: RunMeta = decode_from_std_read(&mut r, cfg)?;
        Ok(Self {
            cfg,
            r,
            meta,
            eof_reached: false,
            last_tick_seen: 0,
            peek: None,
        })
    }

    /// Non-blocking: return all events for `tick`, set eof_reached at EOF.
    pub fn next_for_tick(&mut self, tick: u64) -> Result<Vec<TickEvents>> {
        let mut out = Vec::new();

        if let Some(ev) = self.peek.take() {
            if ev.tick == tick {
                self.last_tick_seen = self.last_tick_seen.max(ev.tick);
                out.push(ev);
            } else {
                self.peek = Some(ev);
                return Ok(out);
            }
        }

        const MAX_READS_PER_CALL: usize = 64;
        for _ in 0..MAX_READS_PER_CALL {
            match decode_from_std_read::<TickEvents, _, _>(&mut self.r, self.cfg) {
                Ok(ev) if ev.tick == tick => {
                    self.last_tick_seen = self.last_tick_seen.max(ev.tick);
                    out.push(ev);
                }
                Ok(ev) => {
                    self.last_tick_seen = self.last_tick_seen.max(ev.tick);
                    self.peek = Some(ev);
                    break;
                }
                Err(_) => {
                    self.eof_reached = true;
                    break;
                }
            }
        }
        Ok(out)
    }
}
