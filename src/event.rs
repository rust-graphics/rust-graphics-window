use log::{log_f, result_f, unwrap_f};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc::{channel, Sender},
        Arc, Mutex, RwLock, Weak,
    },
    thread::{spawn, JoinHandle},
    time::{Duration, Instant},
};

pub type FingerIndexType = i64;

#[derive(Debug)]
pub enum Mouse {
    Left,
    Right,
    Middle,
    Back,
    Forward,
    Offic,
}

#[derive(Debug)]
pub enum Keyboard {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Escape(u8),
    Function(u8),
    PrintScreen,
    ScrollLock,
    PauseBreak,
    BackQuote,
    Number { number: i32, padd: bool },
    Backspace,
    Delete,
    Insert,
    Home,
    End,
    PageUp,
    PageDown,
    NumLock,
    Slash(u8),
    Star,
    Plus(u8),
    Minus(u8),
    Enter(u8),
    Period(u8),
    Tab,
    SquareBracketLeft,
    SquareBracketRight,
    CapseLock,
    SemiColon,
    Quotem,
    BackSlash(u8),
    Shift(u8),
    Comma,
    Control(u8),
    Alt(u8),
    Space(u8),
    Command(u8),
    Super(u8),
    Properties(u8),
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Equal,
    Unknown,
}

#[derive(Debug)]
pub enum Button {
    Mouse(Mouse),
    Keyboard(Keyboard),
}

#[derive(Debug)]
pub enum Window {
    SizeChange {
        width: i64,
        height: i64,
        ratio: f64,
        previous_width: i64,
        previous_height: i64,
        previous_ratio: f64,
        delta_width: i64,
        delta_height: i64,
        delta_ratio: f64,
    },
}

#[derive(Debug)]
pub enum Move {
    Mouse {
        previous: (i64, i64),
        current: (i64, i64),
        delta: (i64, i64),
    },
    Touch {
        index: FingerIndexType,
        previous: (i64, i64),
        current: (i64, i64),
        delta: (i64, i64),
    },
}

#[derive(Debug)]
pub enum ButtonAction {
    Press,
    Release,
}

#[derive(Debug)]
pub enum TouchAction {
    Press,
    HardPress,
    Release,
}

#[derive(Debug)]
pub enum TouchGesture {
    Tap, // todo
    Drag {
        index: FingerIndexType,
        start: (i64, i64),
        previous: (i64, i64),
        current: (i64, i64),
        delta: (i64, i64),
    },
    Scale {
        first: (FingerIndexType, (i64, i64)),
        second: (FingerIndexType, (i64, i64)),
        start: i64,
        previous: i64,
        current: i64,
        delta: i64,
    },
}

#[derive(Debug)]
pub enum GestureState {
    Started,
    InMiddle,
    Ended,
    Canceled,
}

#[derive(Debug)]
pub enum Touch {
    Gesture {
        start_time: Instant,
        duration: Duration,
        state: GestureState,
        gest: TouchGesture,
    },
    Raw {
        index: FingerIndexType,
        action: TouchAction,
        point: (i64, i64),
    },
}

#[derive(Debug)]
pub enum Data {
    Move(Move),
    Button {
        button: Button,
        action: ButtonAction,
    },
    Touch(Touch),
    Window(Window),
    Quit,
    Terminate,
}

#[derive(Debug)]
pub struct Event {
    id: u64,
    time: Instant,
    data: Data,
}

pub static NEXT_ID: AtomicU64 = AtomicU64::new(1);

impl Event {
    pub fn new(data: Data) -> Self {
        Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            time: Instant::now(),
            data,
        }
    }

    pub fn get_id(&self) -> u64 {
        self.id
    }

    pub fn get_time(&self) -> &Instant {
        &self.time
    }

    pub fn get_data(&self) -> &Data {
        &self.data
    }
}

pub trait Listener: Send + Sync {
    fn on_event(&mut self, event: &Event);
}

pub struct Engine {
    listeners: Arc<Mutex<BTreeMap<i64, Vec<Weak<RwLock<dyn Listener>>>>>>,
    processor: Option<JoinHandle<()>>,
    sender: Sender<Event>,
}

impl Engine {
    pub(crate) fn new() -> Self {
        let listeners: Arc<Mutex<BTreeMap<i64, Vec<Weak<RwLock<dyn Listener>>>>>> =
            Arc::new(Mutex::new(BTreeMap::new()));
        let (sender, receiver) = channel();
        let ls = listeners.clone();
        let processor = Some(spawn(move || loop {
            let e: Event = result_f!(receiver.recv());
            match e.get_data() {
                &Data::Terminate => return,
                _ => (),
            }
            let listeners = result_f!(ls.lock());
            for (_, ls) in &*listeners {
                for l in ls {
                    result_f!(unwrap_f!(l.upgrade()).write()).on_event(&e);
                }
            }
        }));
        Self {
            listeners,
            processor,
            sender,
        }
    }

    pub fn add(&self, priority: i64, l: Weak<RwLock<dyn Listener>>) {
        unwrap_f!((*result_f!(self.listeners.lock())).get_mut(&priority)).push(l);
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        if let Some(processor) = self.processor.take() {
            result_f!(processor.join());
        }
    }
}
