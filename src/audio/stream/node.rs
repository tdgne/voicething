use crate::audio::common::{Sample, SampleChunk};
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{channel, sync_channel, Receiver, Sender, SyncSender};
use uuid::Uuid;
#[macro_use]
use enum_dispatch::enum_dispatch;

pub type ChunkSender<S> = Sender<SampleChunk<S>>;

pub type SyncChunkSender<S> = SyncSender<SampleChunk<S>>;

pub type ChunkReceiver<S> = Receiver<SampleChunk<S>>;

pub fn chunk_channel<S: Sample>() -> (ChunkSender<S>, ChunkReceiver<S>) {
    channel()
}

pub fn sync_chunk_channel<S: Sample>(n: usize) -> (SyncChunkSender<S>, ChunkReceiver<S>) {
    sync_channel(n)
}

pub trait HasId {
    fn id(&self) -> Uuid;
}

pub trait SingleInput<S: Sample, T: Sample>: HasId {
    fn input(&self) -> Option<&ChunkReceiver<S>>;

    fn outputs(&self) -> &[SyncChunkSender<T>];

    fn set_input(&mut self, rx: Option<ChunkReceiver<S>>);

    fn add_output(&mut self, tx: SyncChunkSender<T>);

    fn process_chunk(&mut self, chunk: SampleChunk<S>) -> SampleChunk<T>;

    fn run_once(&mut self) {
        if let Some(input) = self.input() {
            if let Some(chunk) = input.try_recv().ok() {
                let chunk = self.process_chunk(chunk);
                for output in self.outputs().iter() {
                    let _ = output.try_send(chunk.clone());
                }
            }
        }
    }
}

use super::*;

macro_rules! define_node {
    ($( ($v:ident: $n:ident $(< $t:ident >)?) ),*) => {
        #[derive(Serialize, Deserialize, Debug)]
        pub enum Node {
            $(
                $v($n$(<$t>)?),
            )*
        }

        impl Node {
            pub fn id(&self) -> Uuid {
                use Node::*;
                match self {
                    $(
                        $v(n) => n.id(),
                    )*
                }
            }

            pub fn run_once(&mut self) {
                use Node::*;
                match self {
                    $(
                        $v(n) => n.run_once(),
                    )*
                }
            }
        }
    }
}

#[macro_export]
macro_rules! operate_connection {
    (match $from:expr, $to:expr, { $( ($f:ident => $($t:ident,)*),)* }, do($ident_f:ident, $ident_t:ident){$do:expr}, err{$err:expr}) => {{
        use Node::*;
        match $from {
            $(
                $f($ident_f) => match $to {
                    $(
                        $t($ident_t) => { $do },
                    )*
                    _ => { $err },
                },
            )*
            _ => { $err },
        }
    }};
    (match $from:expr, $to:expr, do($ident_f:ident, $ident_t:ident){$do:expr}, err{$err:expr}) => {
        operate_connection!(
            match $from, $to, {
                ( Input => Output, Psola, Windower, ),
                ( Psola => Output, Windower, ),
                ( Windower => Psola, Dewindower, ),
                ( Dewindower => Output, Psola, Windower, ),
            }, do($ident_f, $ident_t) {
                $do
            }, err{
                $err
            }
        )
    };
}

define_node!(
    (Psola: PsolaNode),
    (Input: IdentityNode<f32>),
    (Output: IdentityNode<f32>),
    (Windower: Windower<f32>),
    (Dewindower: Dewindower<f32>)
);
