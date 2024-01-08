use std::io::{Read, Write};
use std::mem::size_of;
#[cfg(feature = "serialize_serde")]
use serde::{Deserialize, Serialize};
use atlas_common::crypto::hash::{Context, Digest};
use atlas_common::error::*;
use atlas_common::globals::ReadOnly;
use atlas_common::ordering::SeqNo;

pub struct InstallStateMessage<S> where S: MonolithicState {
    state: S,
}

pub struct AppStateMessage<S> where S: MonolithicState {
    seq: SeqNo,
    state: S,
}

/// The type abstraction for a monolithic state (only needs to be serializable, in reality)
#[cfg(feature = "serialize_serde")]
pub trait MonolithicState: for<'a> Deserialize<'a> + Serialize + Send + Sync + Clone {
    ///Serialize a request from your service, given the writer to serialize into
    ///  (either for network sending or persistent storing)
    fn serialize_state<W>(w: W, request: &Self) -> Result<()> where W: Write;

    ///Deserialize a request that was generated by the serialize request function above
    ///  (either for network sending or persistent storing)
    fn deserialize_state<R>(r: R) -> Result<Self> where R: Read, Self: Sized;

    fn size(&self) -> usize;
}

#[cfg(feature = "serialize_capnp")]
pub trait MonolithicState: Send + Sync + Clone {}


impl<S> AppStateMessage<S> where S: MonolithicState {
    pub fn new(seq: SeqNo, state: S) -> Self {
        AppStateMessage {
            seq,
            state,
        }
    }

    pub fn seq(&self) -> SeqNo {
        self.seq
    }

    pub fn state(&self) -> &S {
        &self.state
    }

    pub fn into_state(self) -> S { self.state }
}

impl<S> InstallStateMessage<S> where S: MonolithicState {
    pub fn new(state: S) -> Self {
        InstallStateMessage {
            state,
        }
    }

    pub fn state(&self) -> &S {
        &self.state
    }

    pub fn into_state(self) -> S {
        self.state
    }
}

pub fn digest_state<S: MonolithicState>(appstate: &S) -> Result<Digest> {
    let mut state_vec = Vec::with_capacity(size_of::<S>());

    S::serialize_state(&mut state_vec, &appstate)?;

    let mut ctx = Context::new();

    ctx.update(&state_vec);

    let digest = ctx.finish();

    Ok(digest)
}